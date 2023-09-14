use std::{env, sync::Arc};
use tokio_postgres::Client;
use tracing::error;

use crate::tls;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
}

impl AppState {
    pub async fn new() -> AppState {
        let connection_string =
            env::var("TDS_FB_CONNECTION_STRING").expect("Failed to read connection string");

        let (client, cx) = tokio_postgres::connect(&connection_string, tls::create_tls_config())
            .await
            .expect("Failed to create database connection");

        tokio::spawn(async move {
            if let Err(e) = cx.await {
                error!("Connection error: {}", e);
            }
        });

        AppState {
            client: Arc::new(client),
        }
    }
}
