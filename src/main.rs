use std::sync::Arc;
use std::time::Duration;

use ::config::Config;
use actix_jwt_auth_middleware::use_jwt::UseJWTOnScope;
use actix_jwt_auth_middleware::{Authority, TokenSigner};
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use ed25519_compact::KeyPair;
use jwt_compact::alg::Ed25519;
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vpn::controller::user_controller::{add_ip_to_peer, get_all_users, keys, login, signup};
use vpn::model::user::UserClaims;
use wiretun::{Cidr, Device, DeviceConfig, PeerConfig};

use config::ExampleConfig;
use vpn::controller::admin_controller::{create_peer, delete_peer, get_all_peers};
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
    let KeyPair {
        pk: public_key,
        sk: secret_key,
    } = KeyPair::generate();

    println!("secret key: {:?}", secret_key);

    HttpServer::new(move || {
        let device = Arc::clone(&device);
        let authority = Authority::<UserClaims, Ed25519, _, _>::new()
            .refresh_authorizer(|| async move { Ok(()) })
            .token_signer(Some(
                TokenSigner::new()
                    .signing_key(secret_key.clone())
                    .access_token_name("my_access_token_name_alan")
                    .refresh_token_lifetime(Duration::from_secs(120 * 60))
                    .algorithm(Ed25519)
                    .build()
                    .expect("wasn't able to create the token signer"),
            ))
            .verifying_key(public_key)
            .build()
            .expect("wasn't able to create the authority");

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(device))
            .service(
                web::scope("/admin")
                    .service(get_all_peers)
                    .service(create_peer)
                    .service(delete_peer),
            )
            .service(
                web::scope("/user").service(signup).service(login).use_jwt(
                    authority,
                    web::scope("")
                        .service(get_all_users)
                        .service(add_ip_to_peer)
                        .service(keys),
                ),
            )
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
