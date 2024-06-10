use actix_jwt_auth_middleware::FromRequest;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")] // singular 'user' is a keyword..
pub struct User {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub password: String,
    pub role: String,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, FromRequest)]
pub struct UserClaims {
    pub id: i64,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserSignUpRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserSignUpResponse {
    pub id: i64,
    pub email: String,
    pub username: String,
}
