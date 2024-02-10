use crate::{
    models::transacao::{Transacao, TransacaoInput},
    utils::app_state::AppState,
};

use actix_web::{web, HttpResponse, Responder};

pub async fn create_transaction(
    cliente_id: i32,
    input: TransacaoInput,
    app_state: &AppState,
) -> Result<Transacao, sqlx::error::Error> {
    let mut db_trx = app_state.pool.begin().await?;
    let transacao = sqlx::query_as::<_, Transacao>("INSERT INTO transacoes(cliente_id, tipo, valor, descricao) values ($1, $2, $3, $4) RETURNING *;")
  .bind(cliente_id)
  .bind(input.tipo)
  .bind(input.valor as i64)
  .bind(input.descricao)
  .fetch_one(&mut *db_trx).await?;
    db_trx.commit().await?;
    Ok(transacao)
}

#[actix_web::post("")]
pub async fn create(
    cliente_id: web::Path<u32>,
    input: web::Json<TransacaoInput>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let input = input.into_inner();
    match create_transaction(cliente_id.into_inner() as i32, input.into(), &app_state).await {
        Ok(transacao) => HttpResponse::Created().json(transacao),
        Err(err) => {
            dbg!(&err);
            match err {
                sqlx::Error::RowNotFound => HttpResponse::NotFound().finish(),
                _ => HttpResponse::InternalServerError().finish(),
            }
        },
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/transacoes").service(create));
}
