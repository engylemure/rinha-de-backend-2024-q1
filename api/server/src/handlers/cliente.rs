use crate::{
    handlers::transacao,
    models::transacao::Transacao,
    utils::app_state::{AppState, PgPoolRunError},
};

use actix_web::{web, HttpResponse, Responder};
use bb8_postgres::tokio_postgres::{types::Type, Row};
use chrono::{DateTime, Utc};

use serde::Serialize;

#[derive(Serialize)]
struct Saldo {
    total: i32,
    data_extrato: DateTime<Utc>,
    limite: i32,
}

impl TryFrom<Row> for Saldo {
    type Error = bb8_postgres::tokio_postgres::Error;

    fn try_from(value: Row) -> Result<Self, Self::Error> {
        Ok(Self {
            total: value.try_get("total")?,
            data_extrato: value.try_get("data_extrato")?,
            limite: value.try_get("limite")?,
        })
    }
}

#[derive(Serialize)]
struct Extrato {
    saldo: Saldo,
    ultimas_transacoes: Vec<Transacao>,
}

async fn get_extrato(
    cliente_id: i32,
    app_state: web::Data<AppState>,
) -> Result<Extrato, PgPoolRunError> {
    let (saldo, transacoes) = {
        let mut conn = app_state.pool.get().await?;
        let trx = conn.transaction().await?;
        let (saldo_stmnt, transacoes_stmnt) = futures_util::try_join!(
            trx.prepare_typed(
                r#"
        SELECT
            saldo as total, limite, NOW() as data_extrato
        FROM
            clientes
        WHERE
            clientes.id = $1
        "#,
                &[Type::INT4],
            ),
            trx.prepare_typed(
                r#"
            SELECT
                tipo, descricao, realizada_em, valor
            FROM 
                transacoes
            WHERE
                cliente_id = $1 ORDER BY realizada_em DESC LIMIT 10
        "#,
                &[Type::INT4]
            )
        )?;
        (
            trx.query_one(&saldo_stmnt, &[&cliente_id]).await?,
            trx.query(&transacoes_stmnt, &[&cliente_id]).await?,
        )
    };
    Ok(Extrato {
        saldo: saldo.try_into()?,
        ultimas_transacoes: transacoes
            .into_iter()
            .map(Transacao::try_from)
            .collect::<Result<_, _>>()?,
    })
}

#[actix_web::get("/extrato")]
pub async fn extrato(cliente_id: web::Path<u32>, app_state: web::Data<AppState>) -> impl Responder {
    match get_extrato(cliente_id.into_inner() as i32, app_state).await {
        Ok(extrato) => HttpResponse::Ok().json(extrato),
        Err(err) => match err {
            bb8::RunError::User(_) => HttpResponse::NotFound().finish(),
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
