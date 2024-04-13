use uuid::Uuid;
use hiivelabs_rand_utils_lib::prelude::create_seed_from_bytes;

///
///
/// # Arguments
///
/// * `guid`:
/// * `x`:
/// * `y`:
///
/// returns: [u8; 16]
pub fn create_seed_from_guid_x_y(guid: Uuid, x: isize, y: isize) -> [u8; 16] {
    // Convert the GUID to byte array
    let uuid_bytes = guid.as_bytes();

    create_seed_from_guid_bytes_x_y(uuid_bytes, x, y)
}

///
///
/// # Arguments
///
/// * `guid_bytes`:
/// * `x`:
/// * `y`:
///
/// returns: [u8; 16]
pub fn create_seed_from_guid_bytes_x_y(guid_bytes: &[u8; 16], x: isize, y: isize) -> [u8; 16] {
    // Convert the GUID and isize values to byte arrays
    let x_bytes = x.to_ne_bytes();
    let y_bytes = y.to_ne_bytes();
    let capacity = guid_bytes.len() + x_bytes.len() + y_bytes.len();
    let mut byte_vec = Vec::with_capacity(capacity);
    byte_vec.extend_from_slice(guid_bytes);
    byte_vec.extend_from_slice(&x_bytes);
    byte_vec.extend_from_slice(&y_bytes);
    create_seed_from_bytes(byte_vec)
}