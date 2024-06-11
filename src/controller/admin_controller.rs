use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

use actix_web::{delete, Error, get, HttpResponse, post, Responder, web};
use deadpool_postgres::Pool;
use wiretun::{Cidr, Device, PeerConfig, UdpTransport};
use crate::errors::pg_errors::MyError;

use crate::models::peers::peer_config::{CreatePeerRequest, PeerDeleteRequest};
use crate::models::peers::peer_mapper::convert_all_peers_to_my_peer_config;
use crate::service::ip_service::get_ips_from_user_id;
use crate::service::user_service::{get_user_by_email, get_users};
use crate::utils::base64utils::{parse_key_str, PrivateKeyRequest};
use crate::utils::tunneling_utils::StubTun;

#[get("/peers")]
async fn get_all_peers(device: web::Data<Arc<Device<StubTun, UdpTransport>>>) -> HttpResponse {
    let device_ref = device.get_ref();
    let wiretun_peers = device_ref.control().config().peers;
    let my_peers = convert_all_peers_to_my_peer_config(wiretun_peers);

    let my_peers_json = serde_json::to_string(&my_peers)
        .map_err(|err| {
            eprintln!("Error serializing peers to JSON: {}", err);
            HttpResponse::InternalServerError().finish()
        })
        .unwrap_or_else(|_| "[]".to_string());

    HttpResponse::Ok().body(my_peers_json)
}

#[get("/users")]
pub async fn get_all_users(db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    get_users(db_pool).await
}

#[delete("/deletePeer")]
async fn delete_peer(
    device: web::Data<Arc<Device<StubTun, UdpTransport>>>,
    req: web::Json<PeerDeleteRequest>,
) -> impl Responder {
    let device_ref = device.get_ref();
    let public_key_str = req.into_inner().public_key;
    let public_key = match parse_key_str(&public_key_str) {
        Ok(key) => key,
        Err(_) => return HttpResponse::BadRequest().body("Invalid public key format"),
    };

    let peers = device_ref.control().config().peers;

    if peers.contains_key(&public_key) {
        device_ref.control().remove_peer(&public_key);
        HttpResponse::Ok().body("Peer deleted successfully")
    } else {
        HttpResponse::NotFound().body("Peer not found")
    }
}

#[post("/createPeer")]
async fn create_peer(
    device: web::Data<Arc<Device<StubTun, UdpTransport>>>,
    db_pool: web::Data<Pool>,
    req: web::Json<CreatePeerRequest>,
) -> Result<HttpResponse, MyError> {
    let email = req.email.clone();

    let user = match get_user_by_email(db_pool.clone(), email).await {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::NotFound().json("User not found")),
    };

    let ips = match get_ips_from_user_id(db_pool.clone(), user.id).await {
        Ok(ips) => ips,
        Err(_) => return Ok(HttpResponse::NotFound().json("IPs not found")),
    };

    let hash_set_ips: HashSet<Cidr> = ips.iter()
        .map(|ip| Cidr::from_str(&ip.ip).unwrap())
        .collect();

    let public_key_str = match user.public_key {
        Some(pk) => pk,
        None => return Ok(HttpResponse::NotFound().json("User does not have a public key")),
    };

    let public_key = match parse_key_str(&public_key_str) {
        Ok(key) => key,
        Err(error_response) => return Ok(error_response),
    };

    let peer_config = PeerConfig {
        public_key,
        allowed_ips: hash_set_ips,
        endpoint: None,
        preshared_key: None,
        persistent_keepalive: None,
    };

    let device_ref = device.get_ref();
    device_ref.control().insert_peer(peer_config);
    Ok(HttpResponse::Ok().json("Peer created successfully"))
}

#[post("/updatePrivateKey")]
async fn update_private_key(device: web::Data<Arc<Device<StubTun, UdpTransport>>>, req: web::Json<PrivateKeyRequest>) -> Result<HttpResponse, MyError> {
    let device_ref = device.get_ref();

    let private_key = match parse_key_str(&req.private_key) {
        Ok(key) => key,
        Err(error_response) => return Ok(error_response),
    };

    device_ref.control().config().private_key = private_key;
    Ok(HttpResponse::Ok().json("Private key updated successfully"))
}