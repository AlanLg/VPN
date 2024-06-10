use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "ips")]
pub struct Ip {
    pub id: i64,
    pub ip: String,
    pub user_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct AddIpRequest {
    pub ip: String,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddIpBdd {
    pub ip: String,
    pub user_id: i64,
}