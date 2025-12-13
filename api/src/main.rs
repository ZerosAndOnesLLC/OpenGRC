use anyhow::Result;
use opengrc_api::{
    cache::CacheClient, config::Config, middleware::AuthState, routes,
    search::SearchClient, services::AppServices, storage::StorageClient,
    utils::EncryptionService, workers::ControlTestingWorker,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,opengrc_api=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Starting OpenGRC API");

    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully");

    let db_pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(Duration::from_secs(config.database.acquire_timeout))
        .connect(config.database_url())
        .await?;

    tracing::info!("Database connection pool established");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;

    tracing::info!("Database migrations completed");

    let cache = CacheClient::new(config.redis_url()).await?;
    tracing::info!("Redis connection established");

    let storage = StorageClient::new(&config.s3).await?;
    tracing::info!("S3 storage client initialized");

    let search = if config.meilisearch.enabled {
        let client = SearchClient::new(
            &config.meilisearch.host,
            config.meilisearch.api_key.as_deref(),
        );
        if let Err(e) = client.init_index().await {
            tracing::warn!("Failed to initialize Meilisearch index: {}. Search will be disabled.", e);
            SearchClient::disabled()
        } else {
            tracing::info!("Meilisearch client initialized at {}", config.meilisearch.host);
            client
        }
    } else {
        tracing::info!("Meilisearch is disabled");
        SearchClient::disabled()
    };

    let encryption = EncryptionService::new(&config.encryption.key)
        .expect("Failed to initialize encryption service - check ENCRYPTION_KEY");
    tracing::info!("Encryption service initialized");

    let services = Arc::new(AppServices::new(
        db_pool,
        cache,
        storage,
        search,
        encryption,
        config.server.api_base_url.clone(),
    ));

    let auth_state = Arc::new(AuthState::new(
        config.titanium_vault.api_url.clone(),
        config.titanium_vault.client_id.clone(),
        config.titanium_vault.client_secret.clone(),
        config.titanium_vault.redirect_uri.clone(),
    ));

    let app = routes::create_router(services.clone(), auth_state, config.cors.origins.clone());

    // Start the control testing worker
    let worker = Arc::new(ControlTestingWorker::new(services.db.clone()));
    tokio::spawn(worker.run());
    tracing::info!("Control testing worker started");

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    tracing::info!("Server shutdown complete");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal");
        },
    }

    tracing::info!("Initiating graceful shutdown");
}
