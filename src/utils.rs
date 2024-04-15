use uuid::Uuid;

pub fn generate_nonce() -> [u8;16] {
    return Uuid::new_v4().into_bytes()
}