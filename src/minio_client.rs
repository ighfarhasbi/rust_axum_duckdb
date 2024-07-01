use std::sync::Arc;

use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::client::ClientBuilder;
use minio::s3::http::BaseUrl;

pub struct MinioClient {
    pub client: Arc<Client>
}

impl MinioClient {
    pub fn new(endpoint: &str, access_key: &str, secret_key: &str) -> Self {
        let base_url = endpoint.parse::<BaseUrl>().unwrap();
        let creds = StaticProvider::new(access_key, secret_key, None);
        let client = ClientBuilder::new(base_url)
            .provider(Some(Box::new(creds)))
            .build()
            .unwrap();
        // let client = ClientBuilder::new(endpoint, creds).unwrap();
        MinioClient {
            client: Arc::new(client),
        }
    }
}