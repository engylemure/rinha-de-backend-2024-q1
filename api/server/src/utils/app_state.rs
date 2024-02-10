use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::utils::env::EnvironmentValues;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

impl AppState {
    pub async fn from(env_values: &EnvironmentValues) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPoolOptions::new()
            .max_connections(env_values.db_pool_max_size)
            .connect(&env_values.database_url)
            .await?;
        Ok(Self { pool })
    }
}
