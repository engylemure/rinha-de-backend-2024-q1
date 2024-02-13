mod handlers;
mod models;
mod utils;

use crate::handlers::cliente;
use crate::utils::app_state::AppState;
use crate::utils::env::{EnvironmentValues, LoggerOutput};
use crate::utils::telemetry;
use actix_cors::Cors;
use actix_web::error::JsonPayloadError;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};
use tracing_actix_web::TracingLogger;

pub async fn server() -> Result<(), Box<dyn std::error::Error>> {
    let env_values = Arc::new(EnvironmentValues::init());
    if let Some(LoggerOutput::Stdout) = env_values.logger {
        telemetry::init()
    }
    let app_state = AppState::from(&env_values).await?;
    let socket: SocketAddr =
        format!("{}:{}", env_values.server_host, env_values.server_port).parse()?;
    tracing::info!("Starting App Server at: {}", socket);
    let app_state = web::Data::new(app_state);
    let json_config =
        web::JsonConfig::default()
            .limit(4096)
            .error_handler(|err, _: &actix_web::HttpRequest| match err {
                JsonPayloadError::Deserialize(err) => {
                    actix_web::error::InternalError::from_response(
                        err,
                        HttpResponse::UnprocessableEntity().into(),
                    )
                    .into()
                }
                err => actix_web::error::InternalError::from_response(
                    err,
                    HttpResponse::BadRequest().into(),
                )
                .into(),
            });
    if env_values.logger.is_none() {
        HttpServer::new(move || {
            App::new()
                .app_data(json_config.clone())
                .app_data(app_state.clone())
                .wrap(Cors::permissive())
                .configure(cliente::config)
        })
        .keep_alive(Duration::from_secs(env_values.keep_alive))
        .bind(&socket)?
        .run()
        .await?;
    } else {
        HttpServer::new(move || {
            App::new()
                .app_data(json_config.clone())
                .app_data(app_state.clone())
                .wrap(Cors::permissive())
                .wrap(TracingLogger::default())
                .configure(cliente::config)
        })
        .keep_alive(Duration::from_secs(env_values.keep_alive))
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
