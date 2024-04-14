use crate::chunk_layer::ChunkLayer;

use crate::chunk_seed_utils::chunk_seed_utils_impl::get_chunk_layer_unique_id;
use hiivelabs_storage_lib::prelude::UniqueId;

impl UniqueId for ChunkLayer {
    fn get_unique_id(&self, mangle: bool) -> String {
        get_chunk_layer_unique_id(self.layer_guid_bytes, mangle)
    }
}
