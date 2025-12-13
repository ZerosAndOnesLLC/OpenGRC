use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::{Credentials, Region},
    presigning::PresigningConfig,
    primitives::ByteStream,
    Client,
};
use std::time::Duration;
use uuid::Uuid;

use crate::config::S3Config;
use crate::utils::{AppError, AppResult};

#[derive(Clone)]
pub struct StorageClient {
    client: Client,
    bucket: String,
}

impl StorageClient {
    pub async fn new(config: &S3Config) -> AppResult<Self> {
        let region = Region::new(config.region.clone());

        let sdk_config = if let (Some(access_key), Some(secret_key)) =
            (&config.access_key_id, &config.secret_access_key)
        {
            // Use explicit credentials if provided
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

            // Support custom endpoint for local development (e.g., MinIO, LocalStack)
            if let Some(endpoint) = &config.endpoint {
                builder = builder.endpoint_url(endpoint);
            }

            builder.load().await
        } else {
            // Use default credential chain (IAM role, env vars, etc.)
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

        let client = Client::from_conf(s3_config_builder.build());

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
        })
    }

    /// Generate the S3 key for an evidence file
    fn evidence_key(org_id: Uuid, evidence_id: Uuid, filename: &str) -> String {
        format!("orgs/{}/evidence/{}/{}", org_id, evidence_id, filename)
    }

    /// Upload a file to S3
    pub async fn upload_evidence(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        filename: &str,
        content_type: &str,
        data: Vec<u8>,
    ) -> AppResult<String> {
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

    /// Download a file from S3
    pub async fn download_evidence(&self, key: &str) -> AppResult<(Vec<u8>, String)> {
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

    /// Generate a presigned URL for downloading (valid for 1 hour)
    pub async fn get_presigned_download_url(&self, key: &str) -> AppResult<String> {
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

    /// Generate a presigned URL for uploading (valid for 15 minutes)
    pub async fn get_presigned_upload_url(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        filename: &str,
        content_type: &str,
    ) -> AppResult<(String, String)> {
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

    /// Delete a file from S3
    pub async fn delete_evidence(&self, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("S3 delete failed: {}", e)))?;

        Ok(())
    }

    /// Check if a file exists
    pub async fn file_exists(&self, key: &str) -> bool {
        self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .is_ok()
    }
}
