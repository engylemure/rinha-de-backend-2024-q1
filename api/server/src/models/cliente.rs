use serde::Serialize;

#[derive(Serialize)]
pub struct Cliente {
    id: i64,
    limite: i32,
    saldo: i128,
}
