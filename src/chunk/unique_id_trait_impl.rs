use hiivelabs_rand_utils_lib::prelude::IDim;
use crate::chunk::Chunk;
use crate::chunk_seed_utils::chunk_seed_utils_impl::get_chunk_unique_id;
use hiivelabs_storage_lib::prelude::UniqueId;

impl UniqueId for Chunk {
    fn get_unique_id(&self, mangle: bool) -> String {
        let (cx, cy) = (
            self.bounds.x as isize / self.bounds.height as isize,
            self.bounds.y as isize / self.bounds.height as isize,
        );
        get_chunk_unique_id(&self.guid_bytes, cx, cy, mangle)
    }
}
