use schnellru::{ByLength, LruMap};

use hiivelabs_storage_lib::prelude::{SqliteStorageContainer, StorageContainer};

use crate::chunk::Chunk;


// TODO! Add in chunk_manager_guid param. add load_chunk_from_store.
// TODO! add storage manager as struct member. modify get to check storage.
// TODO! create tests for ChunkStorageManager
// TODO! consider how to put ChunkStorageManager at top level.
// TODO! need to be able to request chunks by (x, y, z)
// TODO! Add tests for get_chunk_coords_for_hash_index (in layer)

pub(crate) struct ChunkStorageManager {
    chunks: LruMap<(isize, isize), Chunk>,
    lru_cache_size: usize,
}

impl ChunkStorageManager {
    pub(crate) fn new(lru_cache_size: u32) -> Self{
        let chunks = {
            let mut hashmap = LruMap::new(ByLength::new(lru_cache_size));
            hashmap.reserve_or_panic(lru_cache_size as usize + 1); // + 1 for luck.
            hashmap
        };
        Self {
            chunks,
            lru_cache_size: lru_cache_size as usize
        }
    }

    pub(crate) fn get(&mut self, cx: isize, cy:isize) -> Option<&mut Chunk> {
        self.chunks.get(&(cx, cy))
    }

    pub(crate) fn insert(&mut self, cx: isize, cy:isize, chunk: Chunk) {
        if self.chunks.len() == self.lru_cache_size {
            // the cache is full.
            // pop the oldest
            if let Some(((cx, cy), oldest_chunk)) = self.chunks.pop_oldest() {
                // need to store this chunk
                self.save_chunk_to_store(cx, cy, oldest_chunk);
            }
        }
        // insert the new chunk
        self.chunks.insert((cx, cy), chunk);
    }

    pub(crate) fn save_chunk_to_store(&self, cx: isize, cy:isize, chunk: Chunk) {
        let storage = SqliteStorageContainer::new("test.db", true).unwrap();
        let result = storage.save_data_to_package(chunk, true).unwrap();
        let _test = storage.load_data_from_package::<Chunk>(result.as_str());
        let (_cx, cy) = (cx, cy);
        println!("{result}");
        println!()
    }
}