use actix_web::{App, HttpServer, web};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use wiretun::{Cidr, Device, DeviceConfig, PeerConfig};
use vpn::controller::admin_controller::{get_all_peers, hello};
use vpn::utils::base64utils::{local_private_key, peer_public_key};
use vpn::utils::tunneling_utils::StubTun;

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
                .public_key(peer_public_key())
                .allowed_ip("10.0.0.1".parse::<Cidr>().unwrap()),
        );
    let tun = StubTun::new();
    let device = Arc::new(Device::with_udp(tun, cfg).await.unwrap());

    HttpServer::new(move || {
        let device = Arc::clone(&device);
        App::new()
            .app_data(web::Data::new(device))
            .service(
                web::scope("/admin")
                    .service(get_all_peers)
            ).service(
            web::scope("/user")
                .service(hello)
        )
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
