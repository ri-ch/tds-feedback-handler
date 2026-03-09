use deadpool_postgres::{Manager, Pool};
use std::env;

use crate::tls;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
}

impl AppState {
    pub async fn new() -> Result<AppState, anyhow::Error> {
        let connection_string = env::var("TDS_FB_CONNECTION_STRING")?;

        let pg_config: tokio_postgres::Config = connection_string.parse()?;

        let max_size = env::var("DB_POOL_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(16);

        let manager = Manager::new(pg_config, tls::create_tls_config());
        let pool = Pool::builder(manager).max_size(max_size).build()?;

        Ok(AppState { pool })
    }
}
