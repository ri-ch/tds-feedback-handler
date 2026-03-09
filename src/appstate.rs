use deadpool_postgres::{Manager, Pool};
use std::env;

use crate::tls;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
}

impl AppState {
    pub async fn new() -> AppState {
        let connection_string =
            env::var("TDS_FB_CONNECTION_STRING").expect("Failed to read connection string");

        let pg_config: tokio_postgres::Config = connection_string
            .parse()
            .expect("Failed to parse connection string");

        let manager = Manager::new(pg_config, tls::create_tls_config());
        let pool = Pool::builder(manager)
            .max_size(16)
            .build()
            .expect("Failed to create connection pool");

        AppState { pool }
    }
}
