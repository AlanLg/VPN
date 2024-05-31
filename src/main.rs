use std::error::Error;
use std::sync::Arc;

use actix_web::{App, get, HttpResponse, HttpServer, Responder};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use wiretun::{Cidr, Device, DeviceConfig, PeerConfig, Tun};

mod tunneling;
use tunneling::wireguard::StubTun;
mod utils;
use utils::utils::local_private_key;
use crate::utils::utils::peer_public_key;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
                .public_kyey(peer_public_key())
                .allowed_ip("10.0.0.1".parse::<Cidr>().unwrap()),
        );
    let tun = StubTun::new();
    let device = Arc::new(Device::with_udp(tun, cfg).await.unwrap());

    HttpServer::new(move || {
        App::new()
            .data(device.clone())
            .service(hello)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
