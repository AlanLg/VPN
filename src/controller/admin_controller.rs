use std::sync::Arc;

use actix_web::{delete, get, HttpResponse, Responder, web};
use wiretun::{Device, UdpTransport};
use crate::models::peers::peer_config::PeerDeleteRequest;

use crate::models::peers::peer_mapper::convert_all_peers_to_my_peer_config;
use crate::utils::tunneling_utils::StubTun;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

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

/*
#[delete("/deletePeer")]
async fn delete_peer(device: web::Data<Arc<Device<StubTun, UdpTransport>>>, req: web::Json<PeerDeleteRequest>) -> impl Responder {
    let device_ref = device.get_ref();
    device_ref.control().insert_peer();
    let public_key_to_delete = req.public_key;


    // Accéder aux pairs
    let mut peers = device_ref.control().config().peers;

    // Trouver et supprimer le pair avec la clé publique spécifiée
    peers.retain(|peer| peer.public_key != public_key_to_delete);

    // Mise à jour de la configuration du dispositif
    device_ref.control().update_peers(peers).await.unwrap();

    HttpResponse::Ok().json(json!({"status": "success", "deleted_peer_public_key": public_key_to_delete}))
}
 */