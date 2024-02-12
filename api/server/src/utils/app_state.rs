use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::Instrument;

use crate::utils::env::EnvironmentValues;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

impl AppState {
    pub async fn from(env_values: &EnvironmentValues) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPoolOptions::new()
            .max_connections(env_values.db_pool_max_size)
            .min_connections(
                env_values
                    .db_pool_min_size
                    .unwrap_or(env_values.db_pool_max_size / 16),
            )
            .acquire_timeout(Duration::new(4, 0))
            .connect(&env_values.database_url)
            .instrument(tracing::info_span!("database connection"))
            .await?;
        Ok(Self { pool })
    }
}
