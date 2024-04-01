use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use uuid::Uuid;
pub(crate) fn create_seed_from_guid_x_y(guid: Uuid, x: isize, y: isize) -> [u8; 16] {
    // Convert the GUID and usize values to byte arrays
    let uuid_bytes = guid.as_bytes();
    let x_bytes = x.to_ne_bytes();
    let y_bytes = y.to_ne_bytes();
    let capacity = uuid_bytes.len() + x_bytes.len() + y_bytes.len();
    let mut byte_vec = Vec::with_capacity(capacity);
    byte_vec.extend_from_slice(uuid_bytes);
    byte_vec.extend_from_slice(&x_bytes);
    byte_vec.extend_from_slice(&y_bytes);
    create_seed_from_bytes(byte_vec)
}

pub(crate) fn create_seed_from_guid_bytes_x_y(
    guid_bytes: &[u8; 16],
    x: isize,
    y: isize,
) -> [u8; 16] {
    // Convert the GUID and usize values to byte arrays
    let x_bytes = x.to_ne_bytes();
    let y_bytes = y.to_ne_bytes();
    let capacity = guid_bytes.len() + x_bytes.len() + y_bytes.len();
    let mut byte_vec = Vec::with_capacity(capacity);
    byte_vec.extend_from_slice(guid_bytes);
    byte_vec.extend_from_slice(&x_bytes);
    byte_vec.extend_from_slice(&y_bytes);
    create_seed_from_bytes(byte_vec)
}

pub(crate) fn create_seed_from_bytes(byte_vec: Vec<u8>) -> [u8; 16] {
    // Create a Blake2bVar hasher and input the byte vector.
    let mut hasher = Blake2bVar::new(16).unwrap();
    hasher.update(byte_vec.as_ref());
    let mut seed = [0u8; 16];
    hasher.finalize_variable(&mut seed).unwrap();
    seed
}
