use crate::database::postgres;
use crate::errors::pg_errors::MyError;
use actix_web::{web, Error, HttpResponse};
use deadpool_postgres::{Client, Pool};

use crate::model::user::User;

pub async fn get_users(db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;

    let users = postgres::get_users(&client).await?;

    Ok(HttpResponse::Ok().json(users))
}

pub async fn get_user_by_email(db_pool: web::Data<Pool>, email: String) -> Result<User, MyError> {
    let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
    let user = postgres::get_user_by_email(&client, email).await?;
    Ok(user)
}
