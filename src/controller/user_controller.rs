use std::sync::Arc;

use actix_web::{Error, get, HttpResponse, post, Responder, web};
use deadpool_postgres::Pool;
use wiretun::{Cidr, Device, UdpTransport};

use crate::{
    service::user_service::get_users,
    utils::{base64utils::encode_base64, key_generation_utils::generate_keys},
};
use crate::models::peers::peer_config::AddIpRequest;
use crate::service::user_service::get_user_by_email;
use crate::utils::base64utils::parse_public_key_str;
use crate::utils::tunneling_utils::StubTun;

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

#[post("/addIpToPeer")]
async fn add_ip_to_peer(
    device: web::Data<Arc<Device<StubTun, UdpTransport>>>,
    db_pool: web::Data<Pool>,
    req: web::Json<AddIpRequest>
) -> impl Responder {
    let email = req.email.clone();
    let ip = req.ip.clone();

    let user = match get_user_by_email(db_pool.clone(), email).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json("User not found"),
    };

    let public_key_str = user.public_key;
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

        device_ref.control().update_peer_allowed_ips(&public_key, new_allowed_ips);

        HttpResponse::Ok().json("IP added to peer")
    } else {
        HttpResponse::NotFound().json("Peer not found")
    }
}