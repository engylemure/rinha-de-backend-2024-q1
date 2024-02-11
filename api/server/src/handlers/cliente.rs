use crate::{
    handlers::transacao,
    models::transacao::{TipoTransacao, Transacao},
    utils::app_state::AppState,
};

use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize)]
struct Saldo {
    total: i32,
    data_extrato: DateTime<Utc>,
    limite: i32,
}

impl Default for Saldo {
    fn default() -> Self {
        Self {
            total: Default::default(),
            data_extrato: Utc::now(),
            limite: Default::default(),
        }
    }
}

#[derive(Serialize, Default)]
struct Extrato {
    saldo: Saldo,
    ultimas_transacoes: Vec<Transacao>,
}

#[derive(Serialize, FromRow)]
struct SaldoETransacao {
    total: i32,
    limite: i32,
    tipo: Option<TipoTransacao>,
    descricao: Option<String>,
    realizada_em: Option<NaiveDateTime>,
    valor: Option<i32>,
}

impl TryFrom<Vec<SaldoETransacao>> for Extrato {
    type Error = sqlx::error::Error;

    fn try_from(value: Vec<SaldoETransacao>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(sqlx::error::Error::RowNotFound)
        } else {
            Ok(value
                .into_iter()
                .fold(Extrato::default(), |mut acc, saldo_e_transacao| {
                    acc.saldo.limite = saldo_e_transacao.limite;
                    acc.saldo.total = saldo_e_transacao.total;
                    if let (Some(tipo), Some(descricao), Some(realizada_em), Some(valor)) = (
                        saldo_e_transacao.tipo,
                        saldo_e_transacao.descricao,
                        saldo_e_transacao.realizada_em,
                        saldo_e_transacao.valor,
                    ) {
                        acc.ultimas_transacoes.push(Transacao {
                            tipo,
                            descricao,
                            realizada_em,
                            valor,
                        })
                    }
                    acc
                }))
        }
    }
}

async fn get_extrato(
    cliente_id: i32,
    app_state: web::Data<AppState>,
) -> Result<Extrato, sqlx::error::Error> {
    Extrato::try_from(
        sqlx::query_as::<_, SaldoETransacao>(
            r#"
            SELECT
                clientes.saldo as total,
                clientes.limite as limite,
                transacoes.tipo as tipo,
                transacoes.descricao as descricao,
                transacoes.realizada_em as realizada_em,
                transacoes.valor as valor
            FROM 
                clientes
            LEFT JOIN 
                transacoes ON transacoes.cliente_id = clientes.id
            WHERE 
                clientes.id = $1
            ORDER BY transacoes.realizada_em DESC LIMIT 10
        "#,
        )
        .bind(cliente_id)
        .fetch_all(&mut *app_state.pool.acquire().await?)
        .await?,
    )
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
