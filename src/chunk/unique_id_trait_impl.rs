use crate::chunk::Chunk;
use crate::chunk_seed_utils::chunk_seed_utils_impl::get_chunk_unique_id;
use hiivelabs_storage_lib::prelude::UniqueId;

impl UniqueId for Chunk {
    fn get_unique_id(&self, mangle: bool) -> String {
        get_chunk_unique_id(self.guid_bytes, self.bounds.x, self.bounds.y, mangle)
    }
}
