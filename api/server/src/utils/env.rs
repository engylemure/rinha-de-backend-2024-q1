use dotenv::dotenv;
use std::{env, str::FromStr};

#[derive(Debug)]
pub struct EnvironmentValues {
    pub server_host: String,
    pub server_port: u16,
    pub rust_env: String,
    pub logger: Option<LoggerOutput>,
    pub db_pool_max_size: u32,
    pub db_pool_min_size: Option<u32>,
    pub database_url: String,
    pub db_host: String,
    pub db_password: String,
    pub db_name: String,
    pub db_user: String,
}

#[derive(Debug)]
pub enum LoggerOutput {
    Otel,
    Stdout,
}

impl FromStr for LoggerOutput {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "otel" => Ok(Self::Otel),
            "stdout" => Ok(Self::Stdout),
            _ => Err(()),
        }
    }
}

impl EnvironmentValues {
    pub fn init() -> Self {
        if let Err(err) = dotenv() {
            println!("dotenv() error: {:?}", err);
        }
        Self {
            server_host: env::var("SERVER_HOST").unwrap_or("0.0.0.0".into()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| String::from("80"))
                .parse()
                .expect("SERVER_PORT must be a number"),
            rust_env: env::var("RUST_ENV").unwrap_or_else(|_| "dev".into()),
            logger: std::env::var("LOGGER_OUTPUT")
                .ok()
                .and_then(|s| s.parse().ok()),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),

            db_pool_max_size: std::env::var("DATABASE_POOL_MAX_SIZE")
                .map(|s| s.parse().ok())
                .ok()
                .flatten()
                .unwrap_or(256),
            db_pool_min_size: std::env::var("DATABASE_POOL_MIN_SIZE")
                .map(|s| s.parse().ok())
                .ok()
                .flatten(),
            db_host: std::env::var("DB_HOST").expect("DB_HOST must be set"),

            db_user: std::env::var("DB_USER").expect("DB_USER must be set"),

            db_password: std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set"),

            db_name: std::env::var("DB_NAME").expect("DB_NAME must be set"),
        }
    }
}
