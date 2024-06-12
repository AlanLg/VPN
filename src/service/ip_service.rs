use actix_web::web;
use deadpool_postgres::{Client, Pool};

use crate::database::postgres;
use crate::errors::pg_errors::MyError;
use crate::models::ip::Ip;

pub async fn get_ips_from_user_id(db_pool: web::Data<Pool>, user_id: i64) -> Result<Vec<Ip>, MyError> {
    let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
    let ip = postgres::get_ips_from_user_id(&client, user_id).await?;
    Ok(ip)
}
