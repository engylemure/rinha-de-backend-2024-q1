use crate::{models::transacao::TransacaoInput, utils::app_state::AppState};

use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::Row;

#[derive(Serialize, Debug)]
pub struct NewSaldo {
    #[serde(skip_serializing)]
    res: i32,
    limite: i32,
    saldo: i32,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for NewSaldo {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            res: row.try_get("res")?,
            limite: row.try_get("limite")?,
            saldo: row.try_get("new_saldo")?,
        })
    }
}

pub async fn apply_transaction(
    cliente_id: i32,
    input: TransacaoInput,
    app_state: &AppState,
) -> Result<NewSaldo, sqlx::error::Error> {
    let mut db_trx = app_state.pool.begin().await?;
    let new_saldo = sqlx::query_as::<_, NewSaldo>("SELECT * FROM criarTransacao($1, $2, $3, $4)")
        .bind(cliente_id)
        .bind(input.valor)
        .bind(input.tipo)
        .bind(input.descricao)
        .fetch_one(&mut *db_trx)
        .await?;
    db_trx.commit().await?;
    Ok(new_saldo)
}

#[actix_web::post("")]
pub async fn create(
    cliente_id: web::Path<i32>,
    input: web::Json<TransacaoInput>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let input = input.into_inner();
    match apply_transaction(cliente_id.into_inner() as i32, input, &app_state).await {
        Ok(new_saldo) if new_saldo.res == 0 => HttpResponse::Ok().json(new_saldo),
        Ok(new_saldo) if new_saldo.res == -1 => HttpResponse::NotFound().finish(),
        Ok(new_saldo) if new_saldo.res == -2 => HttpResponse::UnprocessableEntity().finish(),
        _ => {
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/transacoes").service(create));
}
