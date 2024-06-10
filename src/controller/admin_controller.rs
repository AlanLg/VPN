use std::sync::Arc;

use actix_web::{delete, get, HttpResponse, post, Responder, web};
use deadpool_postgres::Pool;
use wiretun::{Device, PeerConfig, UdpTransport};

use crate::models::peers::peer_config::{CreatePeerRequest, PeerDeleteRequest};
use crate::models::peers::peer_mapper::convert_all_peers_to_my_peer_config;
use crate::service::user_service::get_user_by_email;
use crate::utils::base64utils::parse_public_key_str;
use crate::utils::tunneling_utils::StubTun;

#[get("/allPeers")]
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

#[delete("/deletePeer")]
async fn delete_peer(device: web::Data<Arc<Device<StubTun, UdpTransport>>>, req: web::Json<PeerDeleteRequest>) -> impl Responder {
    let device_ref = device.get_ref();
    let public_key_str = req.into_inner().public_key;
    let public_key = match parse_public_key_str(&public_key_str) {
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
    req: web::Json<CreatePeerRequest>
) -> impl Responder {
    let email = req.email.clone();

    let user = match get_user_by_email(db_pool.clone(), email).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json("User not found"),
    };

    let public_key_str = user.public_key;
    let public_key = match parse_public_key_str(&public_key_str) {
        Ok(key) => key,
        Err(error_response) => return error_response,
    };

    let peer_config = PeerConfig {
        public_key,
        allowed_ips: Default::default(), // Récupérer l'ip par défaut de la base
        endpoint: None,
        preshared_key: None,
        persistent_keepalive: None,
    };

    let device_ref = device.get_ref();
    device_ref.control().insert_peer(peer_config);
    HttpResponse::Ok().json("Peer created successfully")
}