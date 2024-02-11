use std::fmt;

use chrono::NaiveDateTime;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct Transacao {
    pub tipo: TipoTransacao,
    pub descricao: String,
    pub realizada_em: NaiveDateTime,
    pub valor: i32
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
