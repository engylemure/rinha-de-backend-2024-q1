use crate::{models::transacao::TransacaoInput, utils::app_state::AppState};

use actix_web::{http::header, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct NewSaldo {
    limite: i32,
    saldo: i32,
}

pub async fn apply_transaction(
    cliente_id: i32,
    input: TransacaoInput,
    app_state: &AppState,
) -> Result<NewSaldo, sqlx::error::Error> {
    let mut db_trx = app_state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO transacoes(cliente_id, valor, tipo, descricao) values ($1, $2, $3, $4); ",
    )
    .bind(cliente_id)
    .bind(input.valor as i32)
    .bind(input.tipo)
    .bind(input.descricao)
    .execute(&mut *db_trx)
    .await?;
    let new_saldo = sqlx::query_as::<_, NewSaldo>(
        "SELECT saldo, limite FROM clientes WHERE clientes.id = $1 LIMIT 1;",
    )
    .bind(cliente_id)
    .fetch_one(&mut *db_trx)
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
    match apply_transaction(cliente_id.into_inner() as i32, input.into(), &app_state).await {
        Ok(new_saldo) => HttpResponse::Ok().json(new_saldo),
        Err(err) => match err {
            sqlx::Error::RowNotFound => HttpResponse::NotFound().finish(),
            sqlx::Error::Database(err)
                if err.message() == "Saldo e limite indisponivel para realizar transacao" =>
            {
                HttpResponse::UnprocessableEntity()
                    .append_header(header::ContentType(mime::APPLICATION_JSON))
                    .body(r#"{"message":"Saldo e limite indisponivel para realizar transacao"}"#)
            }
            _ => HttpResponse::InternalServerError().finish(),
        },
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/transacoes").service(create));
}
