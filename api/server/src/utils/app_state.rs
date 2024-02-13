use bb8_postgres::tokio_postgres::{Config, NoTls};
use tracing::Instrument;

use crate::utils::env::EnvironmentValues;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub type PgPool = bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>>;
pub type PgPoolRunError = bb8::RunError<bb8_postgres::tokio_postgres::Error>;

impl AppState {
    pub async fn from(env_values: &EnvironmentValues) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = bb8_postgres::PostgresConnectionManager::new(
            Config::default()
                .user(&env_values.db_user)
                .password(&env_values.db_password)
                .host(&env_values.db_host)
                .dbname(&env_values.db_name)
                .to_owned(),
            NoTls,
        );
        let pool: bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>> = bb8::Pool::builder()
            .max_size(env_values.db_pool_max_size)
            .min_idle(
                env_values
                    .db_pool_min_size
                    .unwrap_or(env_values.db_pool_max_size / 4),
            )
            .build(manager)
            .instrument(tracing::info_span!("database connection"))
            .await?;
        Ok(Self { pool })
    }
}
