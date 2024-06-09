use crate::service::user_service::get_users;
use actix_web::{get, web, Error, HttpResponse};
use deadpool_postgres::Pool;

#[get("/users")]
pub async fn get_all_users(db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    get_users(db_pool).await
}
