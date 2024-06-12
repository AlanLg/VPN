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
    pub public_key: String,
    pub private_key: String,
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
    pub ip: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserSignUpResponse {
    pub email: String,
    pub username: String,
    pub public_key: String,
    pub private_key: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AddUserBdd {
    pub email: String,
    pub username: String,
    pub password: String,
    pub public_key: String,
    pub private_key: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserInformationResponse {
    pub email: String,
    pub user_public_key: String,
    pub user_private_key: String,
    pub device_public_key: String,
}
