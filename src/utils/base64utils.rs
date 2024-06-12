use actix_web::HttpResponse;
use base64::engine::general_purpose::STANDARD as base64Encoding;
use base64::Engine;
use serde::Deserialize;

pub fn encode_base64(s: [u8; 32]) -> String {
    base64Encoding.encode(s)
}

pub fn decode_base64(key_str: &str) -> Result<[u8; 32], HttpResponse> {
    match base64Encoding.decode(key_str) {
        Ok(bytes) => {
            if bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                Ok(key)
            } else {
                Err(HttpResponse::BadRequest().json("Invalid key length"))
            }
        },
        Err(_) => Err(HttpResponse::BadRequest().json("Invalid key format")),
    }
}

pub fn local_private_key() -> Result<[u8; 32], HttpResponse> {
    decode_base64("cCt9aay9r1qflp0OseQQkQ19Zjayx3M3tW9MRqV4aHc=")
}

pub fn peer_public_key() -> Result<[u8; 32], HttpResponse> {
    decode_base64("t2Vc/46ESybZDtMqGZNAPNq2+I9XMFeLZItTxSWvHlU=")
}

#[derive(Deserialize)]
pub struct PrivateKeyRequest {
    pub private_key: String,
}
