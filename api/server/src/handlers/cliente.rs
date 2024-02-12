use crate::{handlers::transacao, models::transacao::Transacao, utils::app_state::AppState};

use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
struct Saldo {
    total: i32,
    data_extrato: DateTime<Utc>,
    limite: i32,
}

#[derive(Serialize)]
struct Extrato {
    saldo: Saldo,
    ultimas_transacoes: Vec<Transacao>,
}

async fn get_extrato(
    cliente_id: i32,
    app_state: web::Data<AppState>,
) -> Result<Extrato, sqlx::error::Error> {
    let mut trx = app_state.pool.begin().await?;
    let saldo = sqlx::query_as::<_, Saldo>(
        r#"
        SELECT
            saldo as total, limite, NOW() as data_extrato
        FROM
            clientes
        WHERE
            clientes.id = $1
        "#,
    )
    .bind(cliente_id)
    .fetch_one(&mut *trx)
    .await?;
    let ultimas_transacoes = sqlx::query_as::<_, Transacao>(
        r#"
            SELECT
                tipo, descricao, realizada_em, valor
            FROM 
                transacoes
            WHERE
                cliente_id = $1 ORDER BY realizada_em DESC LIMIT 10
        "#,
    )
    .bind(cliente_id)
    .fetch_all(&mut *trx)
    .await?;
    Ok(Extrato {
        saldo,
        ultimas_transacoes,
    })
}

#[actix_web::get("/extrato")]
pub async fn extrato(cliente_id: web::Path<u32>, app_state: web::Data<AppState>) -> impl Responder {
    match get_extrato(cliente_id.into_inner() as i32, app_state).await {
        Ok(extrato) => HttpResponse::Ok().json(extrato),
        Err(err) => match err {
            sqlx::Error::RowNotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/clientes/{cliente_id}")
            .service(extrato)
            .configure(transacao::config),
    );
}
