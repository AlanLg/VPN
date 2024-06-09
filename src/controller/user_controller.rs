use crate::{
    service::user_service::get_users,
    utils::{base64utils::encode_base64, key_generation_utils::generate_keys},
};
use actix_web::{get, web, Error, HttpResponse};
use deadpool_postgres::Pool;

#[get("/users")]
pub async fn get_all_users(db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    get_users(db_pool).await
}

#[get("/keys")]
pub async fn keys() -> Result<HttpResponse, Error> {
    let (pub_key, priv_key) = generate_keys();
    println!("{} {}", encode_base64(pub_key), encode_base64(priv_key));
    Ok(HttpResponse::Ok().body("working"))
}
