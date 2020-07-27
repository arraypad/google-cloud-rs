use std::env;
use std::fs::File;
use std::sync::Arc;

use json::json;
use tokio::sync::Mutex;

use crate::authorize::{ApplicationCredentials, MetadataManager, TokenProvider, TokenManager};
use crate::storage::api::bucket::{BucketResource, BucketResources};
use crate::storage::{Bucket, Object, Error};

/// The Cloud Storage client, tied to a specific project.
#[derive(Clone)]
pub struct Client {
    pub(crate) project_name: String,
    pub(crate) client: Arc<reqwest::Client>,
    pub(crate) token_provider: Arc<Mutex<Box<dyn TokenProvider + Send>>>,
}

impl Client {
    #[allow(unused)]
    pub(crate) const DOMAIN_NAME: &'static str = "storage.googleapis.com";
    pub(crate) const ENDPOINT: &'static str = "https://storage.googleapis.com/storage/v1";
    /// Cloud Storage uses a slightly different endpoint for uploads.
    pub(crate) const UPLOAD_ENDPOINT: &'static str =
        "https://storage.googleapis.com/upload/storage/v1";
    pub(crate) const SCOPES: [&'static str; 2] = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/devstorage.full_control",
    ];
    #[allow(dead_code)]
    pub(crate) fn uri(uri: &str) -> String {
        if uri.starts_with('/') {
            format!("{}{}", Client::ENDPOINT, uri)
        } else {
            format!("{}/{}", Client::ENDPOINT, uri)
        }
    }

    /// Create a new client for the specified project.
    ///
    /// Credentials are looked up in the `GOOGLE_APPLICATION_CREDENTIALS` environment variable.
    pub async fn new(project_name: impl Into<String>) -> Result<Client, Error> {
        if let Ok(path) = env::var("GOOGLE_APPLICATION_CREDENTIALS") {
			let file = File::open(path)?;
			let creds = json::from_reader(file)?;
			Client::from_credentials(project_name, creds).await
		} else if let Ok(_) = std::env::var("K_SERVICE") {
			let client = reqwest::Client::builder()
				.build()?;

			Ok(Client {
				client: Arc::new(client),
				project_name: project_name.into(),
				token_provider: Arc::new(Mutex::new(Box::new(MetadataManager::new(
					Client::SCOPES.as_ref(),
				)) as Box<dyn TokenProvider + Send>)),
			})
		} else {
			Err(Error::Auth(crate::error::AuthError::Config("Missing both GOOGLE_APPLICATION_CREDENTIALS and metadata service.".to_string())))
		}
    }

    /// Create a new client for the specified project with custom credentials.
    pub async fn from_credentials(
        project_name: impl Into<String>,
        creds: ApplicationCredentials,
    ) -> Result<Client, Error> {
        // let certificate = reqwest::Certificate::from_pem(TLS_CERTS)?;
        let client = reqwest::Client::builder()
            // .add_root_certificate(certificate)
            .build()?;

        Ok(Client {
            client: Arc::new(client),
            project_name: project_name.into(),
            token_provider: Arc::new(Mutex::new(Box::new(TokenManager::new(
                creds,
                Client::SCOPES.as_ref(),
            )) as Box<dyn TokenProvider + Send>)),
        })
    }

    /// Get a handle to a specific bucket.
    pub async fn bucket(&self, name: &str) -> Result<Bucket, Error> {
        let inner = &self.client;
        let uri = format!("{}/b/{}", Client::ENDPOINT, name);

        let token = self.token_provider.lock().await.token().await?;
        let request = inner
            .get(uri.as_str())
            .header("authorization", token)
            .send();
        let response = request.await?;
        let bucket = response
            .error_for_status()?
            .json::<BucketResource>()
            .await?;

        Ok(Bucket::new(self.clone(), bucket.name))
    }

    /// Get a handle to a specific object.
    pub fn object(&self, bucket: &str, name: &str) -> Object {
        Object::new(self.clone(), bucket, name)
    }

    /// List all existing buckets of the current project.
    pub async fn buckets(&mut self) -> Result<Vec<Bucket>, Error> {
        let inner = &self.client;
        let uri = format!("{}/b", Client::ENDPOINT);

        let token = self.token_provider.lock().await.token().await?;
        let request = inner
            .get(uri.as_str())
            .query(&[("project", self.project_name.as_str())])
            .header("authorization", token)
            .send();
        let response = request.await?;
        let resources = response
            .error_for_status()?
            .json::<BucketResources>()
            .await?;

        let buckets = resources
            .items
            .into_iter()
            .map(|resource| Bucket::new(self.clone(), resource.name))
            .collect();

        Ok(buckets)
    }

    /// Create a new bucket and get a handle to it.
    pub async fn create_bucket(&mut self, name: &str) -> Result<Bucket, Error> {
        let inner = &self.client;
        let uri = format!("{}/b", Client::ENDPOINT);

        let body = json!({
            "kind": "storage#bucket",
            "name": name,
        });
        let token = self.token_provider.lock().await.token().await?;
        let request = inner
            .post(uri.as_str())
            .query(&[("project", self.project_name.as_str())])
            .header("authorization", token)
            .json(&body)
            .send();
        let response = request.await?;
        let bucket = response
            .error_for_status()?
            .json::<BucketResource>()
            .await?;

        Ok(Bucket::new(self.clone(), bucket.name))
    }
}
