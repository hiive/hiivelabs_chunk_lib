use schnellru::{ByLength, LruMap};

use hiivelabs_storage_lib::prelude::{SqliteStorageContainer, StorageContainer};

use crate::chunk::Chunk;


// TODO! Add in chunk_manager_guid param. add load_chunk_from_store.
// TODO! add storage manager as struct member. modify get to check storage.
// TODO! create tests for ChunkStorageManager

pub(crate) struct ChunkStorageManager {
    chunks: LruMap<isize, Chunk>,
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

    pub(crate) fn get(&mut self, chunk_ix: &isize) -> Option<&mut Chunk> {
        self.chunks.get(chunk_ix)
    }

    pub(crate) fn insert(&mut self, chunk_ix: &isize, chunk: Chunk) {
        if self.chunks.len() == self.lru_cache_size {
            // the cache is full.
            // pop the oldest
            if let Some((ix, oldest_chunk)) = self.chunks.pop_oldest() {
                // need to store this chunk
                self.save_chunk_to_store(ix, oldest_chunk);
            }
        }
        // insert the new chunk
        self.chunks.insert(*chunk_ix, chunk);
    }

    pub(crate) fn save_chunk_to_store(&self, chunk_ix: isize, chunk: Chunk) {
        let storage = SqliteStorageContainer::new("test.db", true).unwrap();
        let result = storage.save_data_to_package(chunk, true).unwrap();
        let _test = storage.load_data_from_package::<Chunk>(result.as_str());
        let _chunk_ix = chunk_ix;
        println!("{result}");
        println!()
    }
}