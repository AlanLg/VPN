use std::sync::Arc;

use actix_web::{delete, get, HttpResponse, Responder, web};
use serde_json::json;
use wiretun::{Device, UdpTransport};
use crate::models::peers::peer_config::PeerDeleteRequest;

use crate::models::peers::peer_mapper::convert_all_peers_to_my_peer_config;
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

    let public_key: [u8; 32] = match hex::decode(&public_key_str) {
        Ok(bytes) => {
            if bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                key
            } else {
                return HttpResponse::BadRequest().json("Invalid public key length");
            }
        },
        Err(_) => return HttpResponse::BadRequest().json("Invalid public key format"),
    };

    let peers = device_ref.control().config().peers;

    if peers.contains_key(&public_key) {
        device_ref.control().remove_peer(&public_key);

        let updated_peers = device_ref.control().config().peers;

        let my_peers = convert_all_peers_to_my_peer_config(updated_peers);

        let response = json!({
            "code": 200,
            "success": true,
            "payload": {
                "deleted_peer_public_key": public_key_str,
                "remaining_peers": my_peers
            }
        });

        HttpResponse::Ok().json(response)
    } else {
        let response = json!({
            "code": 404,
            "success": false,
            "message": "Peer not found",
        });

        HttpResponse::NotFound().json(response)
    }
}