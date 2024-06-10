use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct PeerConfig {
    pub public_key: String,
    pub allowed_ips: HashSet<String>,
    pub endpoint: Option<SocketAddr>,
    pub preshared_key: Option<[u8; 32]>,
    pub persistent_keepalive: Option<Duration>,
}

#[derive(Deserialize)]
pub struct PeerDeleteRequest {
    pub public_key: String,
}

#[derive(Deserialize)]
pub struct CreatePeerRequest {
    pub email: String,
}