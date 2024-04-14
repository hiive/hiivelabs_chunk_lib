use hiivelabs_rand_utils_lib::prelude::create_seed_from_bytes;
use uuid::Uuid;

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

fn get_friendly_type_name<T>() -> String {
    let t_name = std::any::type_name::<T>();
    t_name.split("::").last().unwrap_or(t_name).to_string()
}
pub(crate) fn get_chunk_manager_unique_id<T>(guid_bytes: [u8; 16], mangle: bool) -> String {
    let t_name = get_friendly_type_name::<T>();
    let unique_id = get_prefixed_unique_id("cm", guid_bytes, 0, 0, mangle, Some(t_name));
    // log::info!("ChunkManager UniqueId: [{}:({})] -> [{unique_id}]", Uuid::from_bytes(guid_bytes), get_friendly_type_name::<T>());
    unique_id
}

pub(crate) fn get_chunk_layer_unique_id(guid_bytes: [u8; 16], mangle: bool) -> String {
    let unique_id = get_prefixed_unique_id("cl", guid_bytes, 0, 0, mangle, None);
    // log::info!("ChunkLayer UniqueId: [{}] -> [{unique_id}]", Uuid::from_bytes(guid_bytes));
    unique_id
}
pub(crate) fn get_chunk_unique_id(
    guid_bytes: [u8; 16],
    cx: isize,
    cy: isize,
    mangle: bool,
) -> String {
    let unique_id = get_prefixed_unique_id("ch", guid_bytes, cx, cy, mangle, None);
    // log::info!("Chunk UniqueId: [{}:({cx},{cy})] -> [{unique_id}]", Uuid::from_bytes(guid_bytes));
    unique_id
}

fn get_prefixed_unique_id(
    prefix: &str,
    guid_bytes: [u8; 16],
    x: isize,
    y: isize,
    mangle: bool,
    extra_data: Option<String>,
) -> String {
    let guid = Uuid::from_bytes(guid_bytes);
    let mut id = {
        match extra_data {
            None => format!("{guid}_{x:016X}_{y:016X}"),
            Some(data) => {
                format!("{guid}_{data}_{x:016X}_{y:016X}")
            }
        }
    };
    if mangle {
        let mangled = Uuid::from_bytes(create_seed_from_bytes(id.as_bytes().to_vec())).to_string();
        id = format!("{mangled}!m");
    }
    format!("{prefix}-{id}")
}
