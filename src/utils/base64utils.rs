use base64::engine::general_purpose::STANDARD as base64Encoding;
use base64::Engine;

pub fn encode_base64(s: [u8; 32]) -> String {
    base64Encoding.encode(s)
}

pub fn decode_base64(s: &str) -> Vec<u8> {
    base64Encoding.decode(s).unwrap()
}

pub fn local_private_key() -> [u8; 32] {
    decode_base64("cCt9aay9r1qflp0OseQQkQ19Zjayx3M3tW9MRqV4aHc=")
        .try_into()
        .unwrap()
}

pub fn peer_public_key() -> [u8; 32] {
    decode_base64("t2Vc/46ESybZDtMqGZNAPNq2+I9XMFeLZItTxSWvHlU=")
        .try_into()
        .unwrap()
}

