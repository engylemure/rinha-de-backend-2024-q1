use crate::{models::transacao::TransacaoInput, utils::app_state::AppState};

use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct NewSaldo {
    limite: i32,
    saldo: i32,
}

#[derive(Debug)]
pub enum CustomError {
    SemSaldoOuLimite,
    DbErr(sqlx::error::Error),
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for CustomError {}

impl From<sqlx::error::Error> for CustomError {
    fn from(value: sqlx::error::Error) -> Self {
        Self::DbErr(value)
    }
}

pub async fn apply_transaction(
    cliente_id: i32,
    input: TransacaoInput,
    app_state: &AppState,
) -> Result<NewSaldo, CustomError> {
    use crate::models::transacao::TipoTransacao::*;
    let mut db_trx = app_state.pool.begin().await?;
    let valor = input.valor as i32;
    let mut new_saldo = sqlx::query_as::<_, NewSaldo>(
        r#"
        SELECT
            saldo, limite
        FROM
            clientes
        WHERE
            clientes.id = $1 LIMIT 1 FOR UPDATE;
    "#,
    )
    .bind(cliente_id)
    .fetch_one(&mut *db_trx)
    .await?;
    match input.tipo {
        Credito => {
            new_saldo.saldo += valor as i32;
        }
        Debito => {
            if new_saldo.saldo - valor < -new_saldo.limite {
                return Err(CustomError::SemSaldoOuLimite);
            }
            new_saldo.saldo -= valor as i32;
        }
    }
    sqlx::query(
        r#"
        UPDATE 
            clientes
        SET 
            saldo = $2
        WHERE id = $1;
    "#,
    )
    .bind(cliente_id)
    .bind(new_saldo.saldo)
    .execute(&mut *db_trx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO transacoes(cliente_id, valor, tipo, descricao) values ($1, $2, $3, $4);
    "#,
    )
    .bind(cliente_id)
    .bind(input.valor as i32)
    .bind(input.tipo)
    .bind(input.descricao)
    .execute(&mut *db_trx)
    .await?;
    db_trx.commit().await?;
    Ok(new_saldo)
}

#[actix_web::post("")]
pub async fn create(
    cliente_id: web::Path<u32>,
    input: web::Json<TransacaoInput>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let input = input.into_inner();
    match apply_transaction(cliente_id.into_inner() as i32, input, &app_state).await {
        Ok(new_saldo) => HttpResponse::Ok().json(new_saldo),
        Err(CustomError::SemSaldoOuLimite) => HttpResponse::UnprocessableEntity().finish(),
        Err(CustomError::DbErr(sqlx::Error::RowNotFound)) => HttpResponse::NotFound().finish(),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/transacoes").service(create));
}
