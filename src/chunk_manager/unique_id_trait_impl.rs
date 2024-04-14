use crate::prelude::ChunkManager;
use hiivelabs_storage_lib::prelude::UniqueId;

use crate::chunk_seed_utils::chunk_seed_utils_impl::get_chunk_manager_unique_id;

impl<T> UniqueId for ChunkManager<T> {
    fn get_unique_id(&self, mangle: bool) -> String {
        get_chunk_manager_unique_id::<T>(self.manager_guid_bytes, mangle)
    }
}
