use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use config::ExampleConfig;
use dotenvy::dotenv;
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vpn::controller::user_controller::get_all_users;
use wiretun::{Cidr, Device, DeviceConfig, PeerConfig};

use ::config::Config;
use vpn::controller::admin_controller::get_all_peers;
use vpn::utils::base64utils::{local_private_key, peer_public_key};
use vpn::utils::tunneling_utils::StubTun;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = Config::builder()
        .add_source(::config::Environment::default())
        .build()
        .unwrap();

    let config: ExampleConfig = config.try_deserialize().unwrap();

    let pool = config.pg.create_pool(None, NoTls).unwrap();

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
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(device.clone()))
            .service(web::scope("/admin").service(get_all_peers))
            .service(web::scope("/users").service(get_all_users))
    })
    .bind(config.server_addr)?
    .run()
    .await
}

mod config {
    use serde::Deserialize;
    #[derive(Debug, Default, Deserialize)]
    pub struct ExampleConfig {
        pub server_addr: String,
        pub pg: deadpool_postgres::Config,
    }
}
