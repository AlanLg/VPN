use std::sync::Arc;

use actix_jwt_auth_middleware::{AuthResult, TokenSigner};
use actix_web::{get, post, web, Error, HttpResponse, Responder};
use deadpool_postgres::Client;
use deadpool_postgres::Pool;
use jwt_compact::alg::Ed25519;
use wiretun::{Cidr, Device, UdpTransport};

use crate::database::postgres;
use crate::database::postgres::check_email_and_password_valid;
use crate::errors::pg_errors::MyError;
use crate::model::user::UserClaims;
use crate::model::user::UserLoginRequest;
use crate::model::user::UserSignUpRequest;
use crate::models::peers::peer_config::AddIpRequest;
use crate::service::user_service::get_user_by_email;
use crate::utils::base64utils::parse_public_key_str;
use crate::utils::tunneling_utils::StubTun;
use crate::{
    service::user_service::get_users,
    utils::{base64utils::encode_base64, key_generation_utils::generate_keys},
};

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

#[post("/login")]
async fn login(
    req: web::Json<UserLoginRequest>,
    token_signer: web::Data<TokenSigner<UserClaims, Ed25519>>,
    db_pool: web::Data<Pool>,
) -> AuthResult<HttpResponse> {
    let user_info: UserLoginRequest = req.into_inner();

    let client: Client = db_pool.get().await.map_err(MyError::PoolError).unwrap();

    let user = check_email_and_password_valid(&client, user_info).await;
    match user {
        Some(user) => {
            let user = UserClaims {
                id: user.id,
                role: user.role,
            };
            Ok(HttpResponse::Ok()
                .cookie(token_signer.create_access_cookie(&user)?)
                .cookie(token_signer.create_refresh_cookie(&user)?)
                .body("You are now logged in"))
        }
        None => Ok(HttpResponse::Forbidden().into()),
    }
}

#[post("/signup")]
async fn signup(
    req: web::Json<UserSignUpRequest>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let user_info: UserSignUpRequest = req.into_inner();

    let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;

    postgres::add_user(&client, user_info).await;

    Ok(HttpResponse::Ok().json("Signup Successful"))
}

#[post("/addIpToPeer")]
async fn add_ip_to_peer(
    device: web::Data<Arc<Device<StubTun, UdpTransport>>>,
    db_pool: web::Data<Pool>,
    req: web::Json<AddIpRequest>,
) -> impl Responder {
    let email = req.email.clone();
    let ip = req.ip.clone();

    let user = match get_user_by_email(db_pool.clone(), email).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json("User not found"),
    };

    let public_key_str = user.public_key.unwrap();

    let public_key = match parse_public_key_str(&public_key_str) {
        Ok(key) => key,
        Err(error_response) => return error_response,
    };

    let device_ref = device.get_ref();
    let peers = device_ref.control().config().peers;

    if let Some(peer) = peers.get(&public_key) {
        let cidr = match ip.parse::<Cidr>() {
            Ok(cidr) => cidr,
            Err(_) => return HttpResponse::BadRequest().json("Invalid IP format"),
        };

        if peer.allowed_ips.contains(&cidr) {
            return HttpResponse::Ok().json("IP already authorized");
        }

        let mut new_allowed_ips = peer.allowed_ips.clone();
        new_allowed_ips.insert(cidr);

        device_ref
            .control()
            .update_peer_allowed_ips(&public_key, new_allowed_ips);

        HttpResponse::Ok().json("IP added to peer")
    } else {
        HttpResponse::NotFound().json("Peer not found")
    }
}
