mod handlers;
mod models;
mod utils;

use crate::handlers::cliente;
use crate::utils::app_state::AppState;
use crate::utils::env::{EnvironmentValues, LoggerOutput};
use crate::utils::telemetry;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};
use tracing_actix_web::TracingLogger;

pub async fn server() -> Result<(), Box<dyn std::error::Error>> {
    let env_values = Arc::new(EnvironmentValues::init());
    match env_values.logger {
        Some(LoggerOutput::Otel) => telemetry::init_otel(),
        Some(LoggerOutput::Stdout) => telemetry::init(),
        _ => (),
    }
    let app_state = AppState::from(&env_values).await?;
    let socket: SocketAddr = format!("[::]:{}", env_values.server_port).parse()?;
    tracing::info!("Starting App Server at: {}", socket);
    let app_state = web::Data::new(app_state);
    if env_values.logger.is_none() {
        HttpServer::new(move || {
            App::new()
                .app_data(app_state.clone())
                .wrap(Cors::permissive())
                .configure(cliente::config)
        })
        .keep_alive(Duration::from_secs(200))
        .bind(&socket)?
        .run()
        .await?;
    } else {
        HttpServer::new(move || {
            App::new()
                .app_data(app_state.clone())
                .wrap(Cors::permissive())
                .wrap(TracingLogger::default())
                .configure(cliente::config)
        })
        .keep_alive(Duration::from_secs(200))
        .bind(&socket)?
        .run()
        .await?;
    }
    // Ensure all spans have been shipped.
    if let Some(LoggerOutput::Otel) = env_values.logger {
        opentelemetry::global::shutdown_tracer_provider();
    }
    Ok(())
}
