use std::collections::{HashMap, HashSet};
use wiretun::Cidr;
use crate::models::peers::peer_config::PeerConfig;

pub fn convert_to_my_peer_config(wiretun_peer_config: wiretun::PeerConfig) -> PeerConfig {
    let my_peer_config = PeerConfig {
        public_key: wiretun_peer_config.public_key,
        allowed_ips: fetch_ips(wiretun_peer_config.allowed_ips),
        endpoint: wiretun_peer_config.endpoint,
        preshared_key: wiretun_peer_config.preshared_key,
        persistent_keepalive: wiretun_peer_config.persistent_keepalive,
    };
    my_peer_config
}

pub fn convert_all_peers_to_my_peer_config(wiretun_peers: HashMap<[u8; 32], wiretun::PeerConfig>) -> Vec<PeerConfig> {
    wiretun_peers
        .into_iter()
        .map(|(_, wiretun_peer_config)| convert_to_my_peer_config(wiretun_peer_config))
        .collect()
}

fn fetch_ips(peers_cidr: HashSet<Cidr>) -> HashSet<String> {
    let mut peers_ips = HashSet::new();

    for cidr in peers_cidr {
        peers_ips.insert(cidr.to_string());
    }

    peers_ips
}