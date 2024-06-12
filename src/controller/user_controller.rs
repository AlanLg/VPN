use std::sync::Arc;

use actix_jwt_auth_middleware::{AuthResult, TokenSigner};
use actix_web::{Error, get, HttpResponse, post, web};
use deadpool_postgres::Client;
use deadpool_postgres::Pool;
use jwt_compact::alg::Ed25519;
use wiretun::{Cidr, Device, UdpTransport};
use wiretun::noise::crypto::PublicKey;

use crate::database::postgres;
use crate::database::postgres::check_email_and_password_valid;
use crate::errors::pg_errors::MyError;
use crate::models::ip::{AddIpBdd, AddIpRequest};
use crate::models::user::{AddUserBdd, UserClaims, UserInformationResponse};
use crate::models::user::UserLoginRequest;
use crate::models::user::UserSignUpRequest;
use crate::service::user_service::{get_user_by_email, get_user_by_id};
use crate::utils::{base64utils::encode_base64, key_generation_utils::generate_keys};
use crate::utils::base64utils::decode_base64;
use crate::utils::tunneling_utils::StubTun;

#[get("/user/keys")]
pub async fn keys() -> Result<HttpResponse, Error> {
    let (pub_key, priv_key) = generate_keys();
    println!("{} {}", encode_base64(pub_key), encode_base64(priv_key));
    Ok(HttpResponse::Ok().body("working"))
}

#[get("/user/informations")]
pub async fn get_necessary_informations(
    device: web::Data<Arc<Device<StubTun, UdpTransport>>>,
    user_claims: UserClaims,
    db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let user = match get_user_by_id(db_pool.clone(), user_claims.id).await {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::NotFound().json("User not found")),
    };

    let device_ref = device.get_ref();
    let device_public_key_str = PublicKey::from(device_ref.control().config().private_key);

    let user_response: UserInformationResponse = UserInformationResponse {
        email: user.email,
        user_public_key: user.public_key,
        user_private_key: user.private_key,
        device_public_key: encode_base64(device_public_key_str.as_bytes().clone()),
    };

    Ok(HttpResponse::Ok().json(user_response))
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

    let (public_key, private_key) = generate_keys();

    let user_bdd = AddUserBdd {
        email: user_info.email,
        username: user_info.username,
        password: user_info.password,
        public_key: encode_base64(public_key),
        private_key: encode_base64(private_key),
    };

    postgres::add_user(&client, user_bdd.clone()).await;

    let user = match get_user_by_email(db_pool.clone(), user_bdd.email.clone()).await {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::NotFound().json("User not found")),
    };

    let ip_bdd = AddIpBdd {
        ip: user_info.ip,
        user_id: user.id,
    };

    postgres::add_ip(&client, ip_bdd).await;

    Ok(HttpResponse::Ok().json("Signup Successful"))
}

#[post("/user/addIpToPeer")]
async fn add_ip_to_peer(
    device: web::Data<Arc<Device<StubTun, UdpTransport>>>,
    db_pool: web::Data<Pool>,
    req: web::Json<AddIpRequest>,
) -> Result<HttpResponse, MyError> {
    let email = req.email.clone();
    let ip = req.ip.clone();

    let user = match get_user_by_email(db_pool.clone(), email).await {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::NotFound().json("User not found")),
    };

    let public_key_str = user.public_key;

    let public_key = match decode_base64(&public_key_str) {
        Ok(key) => key,
        Err(error_response) => return Ok(error_response),
    };

    let device_ref = device.get_ref();
    let peers = device_ref.control().config().peers;

    let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;

    if let Some(peer) = peers.get(&public_key) {
        let cidr = match ip.parse::<Cidr>() {
            Ok(cidr) => cidr,
            Err(_) => return Ok(HttpResponse::BadRequest().json("Invalid IP format")),
        };

        if peer.allowed_ips.contains(&cidr) {
            return Ok(HttpResponse::Ok().json("IP already authorized"));
        }

        let mut new_allowed_ips = peer.allowed_ips.clone();
        new_allowed_ips.insert(cidr);

        let ip_info = AddIpBdd {
            ip: req.ip.clone(),
            user_id: user.id,
        };
        postgres::add_ip(&client, ip_info).await;

        device_ref
            .control()
            .update_peer_allowed_ips(&public_key, new_allowed_ips);

        Ok(HttpResponse::Ok().json("IP added to peer"))
    } else {
        Ok(HttpResponse::NotFound().json("Peer not found"))
    }
}
