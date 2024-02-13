use crate::{
    models::transacao::TransacaoInput,
    utils::app_state::{AppState, PgPoolRunError},
};

use actix_web::{http::header, web, HttpResponse, Responder};
use bb8_postgres::tokio_postgres::{error::SqlState, Row};
use postgres_types::Type;
use serde::Serialize;

#[derive(Serialize)]
pub struct NewSaldo {
    limite: i32,
    saldo: i32,
}

impl TryFrom<Row> for NewSaldo {
    type Error = bb8_postgres::tokio_postgres::Error;

    fn try_from(value: Row) -> Result<Self, Self::Error> {
        Ok(Self {
            limite: value.try_get("limite")?,
            saldo: value.try_get("saldo")?,
        })
    }
}

pub async fn apply_transaction(
    cliente_id: i32,
    input: TransacaoInput,
    app_state: &AppState,
) -> Result<NewSaldo, PgPoolRunError> {
    let mut conn = app_state.pool.get().await?;
    let trx = conn.transaction().await?;
    let (insert_stmnt, saldo_stmnt) = futures_util::try_join!(
        trx.prepare(
            "INSERT INTO transacoes(cliente_id, valor, tipo, descricao) values ($1, $2, $3, $4); ",
        ),
        trx.prepare_typed(
            r#"
        SELECT
            saldo, limite
        FROM
            clientes
        WHERE
            clientes.id = $1 LIMIT 1;
    "#,
            &[Type::INT4]
        )
    )?;
    trx.execute(
        &insert_stmnt,
        &[
            &cliente_id,
            &(input.valor as i32),
            &input.tipo,
            &input.descricao,
        ],
    )
    .await?;
    let new_saldo = trx
        .query_one(&saldo_stmnt, &[&cliente_id])
        .await?
        .try_into()?;
    trx.commit().await?;
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
        Err(err) => {
            match err {
                bb8::RunError::User(err) => {
                    if err
                        .code()
                        .map(|code| &SqlState::NOT_NULL_VIOLATION == code)
                        .unwrap_or_default()
                    {
                        HttpResponse::NotFound().finish()
                    } else if err
                        .as_db_error()
                        .map(|err| {
                            err.message() == "Saldo e limite indisponivel para realizar transacao"
                        })
                        .unwrap_or_default()
                    {
                        HttpResponse::UnprocessableEntity()
                        .append_header(header::ContentType(mime::APPLICATION_JSON))
                        .body(r#"{"message":"Saldo e limite indisponivel para realizar transacao"}"#)
                    } else {
                        HttpResponse::InternalServerError().finish()
                    }
                }
                _ => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/transacoes").service(create));
}
