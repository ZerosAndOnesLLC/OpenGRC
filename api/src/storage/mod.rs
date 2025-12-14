use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::config::StorageConfig;
use crate::utils::{AppError, AppResult};

/// Storage client that abstracts over local filesystem or S3 storage
#[derive(Clone)]
pub struct StorageClient {
    backend: StorageBackend,
}

#[derive(Clone)]
enum StorageBackend {
    Local(LocalStorage),
    S3(S3Storage),
}

// ==================== Local Storage ====================

#[derive(Clone)]
struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn file_path(&self, org_id: Uuid, evidence_id: Uuid, filename: &str) -> PathBuf {
        self.base_path
            .join("orgs")
            .join(org_id.to_string())
            .join("evidence")
            .join(evidence_id.to_string())
            .join(filename)
    }

    async fn ensure_dir(&self, path: &PathBuf) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::InternalServerError(format!("Failed to create directory: {}", e)))?;
        }
        Ok(())
    }

    async fn upload(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        filename: &str,
        _content_type: &str,
        data: Vec<u8>,
    ) -> AppResult<String> {
        let file_path = self.file_path(org_id, evidence_id, filename);
        self.ensure_dir(&file_path).await?;

        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to create file: {}", e)))?;

        file.write_all(&data)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to write file: {}", e)))?;

        // Return relative key (same format as S3)
        let key = format!("orgs/{}/evidence/{}/{}", org_id, evidence_id, filename);
        Ok(key)
    }

    async fn download(&self, key: &str) -> AppResult<(Vec<u8>, String)> {
        let file_path = self.base_path.join(key);

        let data = fs::read(&file_path)
            .await
            .map_err(|e| AppError::NotFound(format!("File not found: {}", e)))?;

        // Guess content type from extension
        let content_type = mime_guess::from_path(&file_path)
            .first_or_octet_stream()
            .to_string();

        Ok((data, content_type))
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let file_path = self.base_path.join(key);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .await
                .map_err(|e| AppError::InternalServerError(format!("Failed to delete file: {}", e)))?;
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> bool {
        let file_path = self.base_path.join(key);
        file_path.exists()
    }

    fn get_download_url(&self, key: &str) -> AppResult<String> {
        // For local storage, return a path that the API can serve
        // The actual serving is handled by a separate endpoint
        Ok(format!("/api/v1/storage/download/{}", key))
    }

    fn get_upload_url(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        filename: &str,
    ) -> AppResult<(String, String)> {
        let key = format!("orgs/{}/evidence/{}/{}", org_id, evidence_id, filename);
        // For local storage, uploads go through the API
        let upload_url = format!("/api/v1/storage/upload/{}", key);
        Ok((upload_url, key))
    }
}

// ==================== S3 Storage ====================

#[derive(Clone)]
struct S3Storage {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl S3Storage {
    async fn new(config: &StorageConfig) -> AppResult<Self> {
        use aws_config::BehaviorVersion;
        use aws_sdk_s3::config::{Credentials, Region};

        let bucket = config.bucket.clone().ok_or_else(|| {
            AppError::InternalServerError("S3_BUCKET must be set when using S3 storage".to_string())
        })?;

        let region = Region::new(config.region.clone());

        let sdk_config = if let (Some(access_key), Some(secret_key)) =
            (&config.access_key_id, &config.secret_access_key)
        {
            let credentials = Credentials::new(
                access_key,
                secret_key,
                None,
                None,
                "opengrc",
            );

            let mut builder = aws_config::defaults(BehaviorVersion::latest())
                .region(region.clone())
                .credentials_provider(credentials);

            if let Some(endpoint) = &config.endpoint {
                builder = builder.endpoint_url(endpoint);
            }

            builder.load().await
        } else {
            let mut builder = aws_config::defaults(BehaviorVersion::latest())
                .region(region.clone());

            if let Some(endpoint) = &config.endpoint {
                builder = builder.endpoint_url(endpoint);
            }

            builder.load().await
        };

        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&sdk_config);

        // Force path style for custom endpoints (MinIO, LocalStack)
        if config.endpoint.is_some() {
            s3_config_builder = s3_config_builder.force_path_style(true);
        }

        let client = aws_sdk_s3::Client::from_conf(s3_config_builder.build());

        Ok(Self { client, bucket })
    }

    fn evidence_key(org_id: Uuid, evidence_id: Uuid, filename: &str) -> String {
        format!("orgs/{}/evidence/{}/{}", org_id, evidence_id, filename)
    }

    async fn upload(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        filename: &str,
        content_type: &str,
        data: Vec<u8>,
    ) -> AppResult<String> {
        use aws_sdk_s3::primitives::ByteStream;

        let key = Self::evidence_key(org_id, evidence_id, filename);
        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .content_type(content_type)
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("S3 upload failed: {}", e)))?;

        Ok(key)
    }

    async fn download(&self, key: &str) -> AppResult<(Vec<u8>, String)> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::NotFound(format!("File not found: {}", e)))?;

        let content_type = response
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read file: {}", e)))?
            .into_bytes()
            .to_vec();

        Ok((data, content_type))
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("S3 delete failed: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> bool {
        self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .is_ok()
    }

    async fn get_presigned_download_url(&self, key: &str) -> AppResult<String> {
        use aws_sdk_s3::presigning::PresigningConfig;

        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(3600))
            .map_err(|e| AppError::InternalServerError(format!("Presigning config error: {}", e)))?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to generate presigned URL: {}", e)))?;

        Ok(presigned.uri().to_string())
    }

    async fn get_presigned_upload_url(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        filename: &str,
        content_type: &str,
    ) -> AppResult<(String, String)> {
        use aws_sdk_s3::presigning::PresigningConfig;

        let key = Self::evidence_key(org_id, evidence_id, filename);

        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(900))
            .map_err(|e| AppError::InternalServerError(format!("Presigning config error: {}", e)))?;

        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to generate presigned URL: {}", e)))?;

        Ok((presigned.uri().to_string(), key))
    }
}

// ==================== StorageClient Implementation ====================

impl StorageClient {
    pub async fn new(config: &StorageConfig) -> AppResult<Self> {
        let backend = if config.is_local() {
            let base_path = config
                .local_path
                .clone()
                .unwrap_or_else(|| "./storage".to_string());

            let path = PathBuf::from(&base_path);

            // Create base directory if it doesn't exist
            if !path.exists() {
                fs::create_dir_all(&path)
                    .await
                    .map_err(|e| AppError::InternalServerError(format!("Failed to create storage directory: {}", e)))?;
            }

            tracing::info!("Using local storage at: {}", path.display());
            StorageBackend::Local(LocalStorage::new(path))
        } else {
            tracing::info!("Using S3 storage: bucket={:?}", config.bucket);
            StorageBackend::S3(S3Storage::new(config).await?)
        };

        Ok(Self { backend })
    }

    /// Upload a file to storage
    pub async fn upload_evidence(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        filename: &str,
        content_type: &str,
        data: Vec<u8>,
    ) -> AppResult<String> {
        match &self.backend {
            StorageBackend::Local(storage) => {
                storage.upload(org_id, evidence_id, filename, content_type, data).await
            }
            StorageBackend::S3(storage) => {
                storage.upload(org_id, evidence_id, filename, content_type, data).await
            }
        }
    }

    /// Download a file from storage
    pub async fn download_evidence(&self, key: &str) -> AppResult<(Vec<u8>, String)> {
        match &self.backend {
            StorageBackend::Local(storage) => storage.download(key).await,
            StorageBackend::S3(storage) => storage.download(key).await,
        }
    }

    /// Delete a file from storage
    pub async fn delete_evidence(&self, key: &str) -> AppResult<()> {
        match &self.backend {
            StorageBackend::Local(storage) => storage.delete(key).await,
            StorageBackend::S3(storage) => storage.delete(key).await,
        }
    }

    /// Check if a file exists
    pub async fn file_exists(&self, key: &str) -> bool {
        match &self.backend {
            StorageBackend::Local(storage) => storage.exists(key).await,
            StorageBackend::S3(storage) => storage.exists(key).await,
        }
    }

    /// Get a URL for downloading a file
    /// For S3: returns a presigned URL
    /// For local: returns an API path that serves the file
    pub async fn get_presigned_download_url(&self, key: &str) -> AppResult<String> {
        match &self.backend {
            StorageBackend::Local(storage) => storage.get_download_url(key),
            StorageBackend::S3(storage) => storage.get_presigned_download_url(key).await,
        }
    }

    /// Get a URL for uploading a file
    /// For S3: returns a presigned URL
    /// For local: returns an API path for upload
    pub async fn get_presigned_upload_url(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        filename: &str,
        content_type: &str,
    ) -> AppResult<(String, String)> {
        match &self.backend {
            StorageBackend::Local(storage) => storage.get_upload_url(org_id, evidence_id, filename),
            StorageBackend::S3(storage) => {
                storage.get_presigned_upload_url(org_id, evidence_id, filename, content_type).await
            }
        }
    }

    /// Returns true if using local storage
    pub fn is_local(&self) -> bool {
        matches!(&self.backend, StorageBackend::Local(_))
    }

    /// Returns true if using S3 storage
    pub fn is_s3(&self) -> bool {
        matches!(&self.backend, StorageBackend::S3(_))
    }
}
