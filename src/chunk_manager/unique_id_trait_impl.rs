use crate::prelude::ChunkManager;
use hiivelabs_rand_utils_lib::prelude::create_seed_from_bytes;
use hiivelabs_storage_lib::prelude::UniqueId;
use std::any::type_name;
use uuid::Uuid;

impl<T> UniqueId for ChunkManager<T> {
    fn get_unique_id(&self, mangle: bool) -> String {
        let guid = Uuid::from_bytes(self.guid_bytes);
        let t_name = type_name::<T>();
        let t_name = t_name.split("::").last().unwrap_or(t_name).to_string();
        let mut id = format!("{guid}_{t_name}");
        if mangle {
            let mangled =
                Uuid::from_bytes(create_seed_from_bytes(id.as_bytes().to_vec())).to_string();
            id = format!("{mangled}!m");
        }
        format!("cm-{id}")
    }
}
