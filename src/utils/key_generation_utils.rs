use x25519_dalek::{PublicKey, StaticSecret};

pub fn generate_keys() -> ([u8; 32], [u8; 32]) {
    let priv_key = StaticSecret::random();
    let pub_key = PublicKey::from(&priv_key);
    (pub_key.to_bytes(), priv_key.to_bytes())
}
