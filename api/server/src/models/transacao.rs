use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

#[derive(Serialize)]
pub struct Transacao {
    id: i32,
    cliente_id: i32,
    tipo: TipoTransacao,
    descricao: String,
    realizada_em: NaiveDateTime,
}

impl FromRow<'_, PgRow> for Transacao {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            cliente_id: row.try_get("cliente_id")?,
            tipo: row.try_get("tipo")?,
            descricao: row.try_get("descricao")?,
            realizada_em: row.try_get("realizada_em")?,
        })
    }
}

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "tipoTransacao")]
pub enum TipoTransacao {
    #[serde(rename = "c")]
    #[sqlx(rename = "c")]
    Credito,
    #[sqlx(rename = "d")]
    #[serde(rename = "d")]
    Debito,
}

#[derive(Deserialize)]
pub struct TransacaoInput {
    pub tipo: TipoTransacao,
    pub descricao: String,
    pub valor: u64,
}
