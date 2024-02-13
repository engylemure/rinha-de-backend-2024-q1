use std::fmt;

use bb8_postgres::tokio_postgres::Row;
use chrono::{DateTime, Utc};
use postgres_types::{FromSql, ToSql};
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};

#[derive(Serialize)]
pub struct Transacao {
    pub tipo: TipoTransacao,
    pub descricao: String,
    pub realizada_em: DateTime<Utc>,
    pub valor: i32,
}

impl TryFrom<Row> for Transacao {
    type Error = bb8_postgres::tokio_postgres::Error;

    fn try_from(value: Row) -> Result<Self, Self::Error> {
        Ok(Self {
            tipo: value.try_get("tipo")?,
            descricao: value.try_get("descricao")?,
            realizada_em: value.try_get("realizada_em")?,
            valor: value.try_get("valor")?,
        })
    }
}

#[derive(Serialize, Deserialize, ToSql, FromSql, Debug)]
#[postgres(name = "tipo_transacao")]
pub enum TipoTransacao {
    #[postgres(name = "c")]
    #[serde(rename = "c")]
    Credito,
    #[serde(rename = "d")]
    #[postgres(name = "d")]
    Debito,
}

#[derive(Deserialize)]
pub struct TransacaoInput {
    pub tipo: TipoTransacao,
    #[serde(deserialize_with = "deserialize_descricao")]
    pub descricao: String,
    pub valor: u32,
}

struct DeserializeDescricao;

impl<'de> de::Visitor<'de> for DeserializeDescricao {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string with minimum of 1 and maximum of 10 characters")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if !v.is_empty() && v.len() <= 10 {
            Ok(v.to_string())
        } else {
            Err(E::invalid_value(Unexpected::Str(v), &self))
        }
    }
}

fn deserialize_descricao<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(DeserializeDescricao)
}
