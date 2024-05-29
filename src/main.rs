use base64::engine::general_purpose::STANDARD as base64Encoding;
use base64::Engine;
use std::error::Error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use wireguard::wireguard::StubTun;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use wiretun::{uapi, Cidr, Device, DeviceConfig, PeerConfig, Tun, TunError};

mod wireguard;

fn decode_base64(s: &str) -> Vec<u8> {
    base64Encoding.decode(s).unwrap()
}

fn local_private_key() -> [u8; 32] {
    decode_base64("cCt9aay9r1qflp0OseQQkQ19Zjayx3M3tW9MRqV4aHc=")
        .try_into()
        .unwrap()
}

fn peer_public_key() -> [u8; 32] {
    decode_base64("t2Vc/46ESybZDtMqGZNAPNq2+I9XMFeLZItTxSWvHlU=")
        .try_into()
        .unwrap()
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = HttpServer::new(|| App::new().service(hello))
        .bind(("127.0.0.1", 8080))?
        .run();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = DeviceConfig::default()
        .listen_port(51820)
        .private_key(local_private_key())
        .peer(
            PeerConfig::default()
                .public_key(peer_public_key())
                .allowed_ip("10.0.0.1".parse::<Cidr>()?),
        );
    let tun = StubTun::new();
    let device = Device::with_udp(tun, cfg).await.unwrap();

    let ctrl = device.control();
    tokio::spawn(uapi::bind_and_handle(ctrl));

    HttpServer::new(move || App::new().service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await;
    Ok(())
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
