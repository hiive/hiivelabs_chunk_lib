use crate::chunk_layer::ChunkLayer;
use hiivelabs_rand_utils_lib::prelude::create_seed_from_bytes;
use hiivelabs_storage_lib::prelude::UniqueId;
use uuid::Uuid;

impl UniqueId for ChunkLayer {
    fn get_unique_id(&self, mangle: bool) -> String {
        let mut id = Uuid::from_bytes(self.guid_bytes).to_string();
        if mangle {
            let mangled =
                Uuid::from_bytes(create_seed_from_bytes(id.as_bytes().to_vec())).to_string();
            id = format!("{mangled}!m");
        }
        format!("cl-{id}")
    }
}
