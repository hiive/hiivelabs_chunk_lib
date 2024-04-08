use crate::chunk::Chunk;
use hiivelabs_rand_utils_lib::prelude::create_seed_from_bytes;
use hiivelabs_storage_lib::prelude::UniqueId;
use uuid::Uuid;

impl UniqueId for Chunk {
    fn get_unique_id(&self, mangle: bool) -> String {
        let guid = Uuid::from_bytes(self.guid_bytes);
        let x = self.bounds.x;
        let y = self.bounds.y;

        let mut id = format!("{guid}_{x:016X}_{y:016X}");
        if mangle {
            let mangled =
                Uuid::from_bytes(create_seed_from_bytes(id.as_bytes().to_vec())).to_string();
            id = format!("{mangled}!m");
        }
        format!("ch-{id}")
    }
}
