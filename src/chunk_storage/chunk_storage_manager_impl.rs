use schnellru::{ByLength, LruMap};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use std::sync::mpsc::{self, Sender};

use crate::chunk::Chunk;
use crate::chunk_seed_utils::chunk_seed_utils_impl::{
    create_seed_from_guid_x_y, get_chunk_unique_id,
};
use crate::chunk_storage::chunk_storage_thread_handler_impl::ChunkStorageThreadHandler;

// TODO! Add in chunk_manager_guid param. add load_chunk_from_store.
// TODO! add storage manager as struct member. modify get to check storage.
// TODO! create tests for ChunkStorageManager
// TODO! consider how to put ChunkStorageManager at top level.
// TODO! need to be able to request chunks by (x, y, z), where z could be layer guid
// TODO! Add tests for get_chunk_coords_for_hash_index (in layer)

pub(crate) struct ChunkStorageManager {
    chunks: LruMap<(isize, isize), Chunk>,
    lru_cache_size: usize,
    owning_manager_guid: Uuid,
    owning_layer_guid: Uuid,
    chunk_storage_thread_handler: Arc<Mutex<ChunkStorageThreadHandler>>,
    storage_tx: Option<Sender<Chunk>>,
}

impl Drop for ChunkStorageManager {
    fn drop(&mut self) {
        // First, we should tell the storage thread to stop running.
        // This could be done by sending a special shutdown signal or closing the channel.
        if let Some(storage_tx) = self.storage_tx.take() {
            drop(storage_tx); // Dropping the sender will close the channel.
        }

        // Now, join the thread to make sure it finishes cleanly.
        //if let thread_handler = self.chunk_storage_thread_handler {
        //     match self.chunk_storage_thread_handler.join() {
        //         Ok(_) => println!("Storage thread has finished."),
        //         Err(e) => println!("Failed to join storage thread: {e:?}"),
        //     }
        //}
        let mut chunk_storage_thread_handler = self.chunk_storage_thread_handler.lock().unwrap();
        chunk_storage_thread_handler.join();
    }
}

impl ChunkStorageManager {
    pub(crate) fn new(
        lru_cache_size: u32,
        owning_manager_guid_bytes: [u8; 16],
        owning_layer_guid_bytes: [u8; 16],
    ) -> Self {
        let chunks = {
            let mut hashmap = LruMap::new(ByLength::new(lru_cache_size));
            hashmap.reserve_or_panic(lru_cache_size as usize + 1); // + 1 for luck.
            hashmap
        };
        let (storage_tx, storage_rx) = mpsc::channel();
        let owning_manager_guid = Uuid::from_bytes(owning_manager_guid_bytes);
        let chunk_storage_thread_handler =
            ChunkStorageThreadHandler::start(owning_manager_guid, storage_rx);

        let owning_layer_guid = Uuid::from_bytes(owning_layer_guid_bytes);
        Self {
            chunks,
            lru_cache_size: lru_cache_size as usize,
            owning_manager_guid,
            owning_layer_guid,
            chunk_storage_thread_handler,
            storage_tx: Some(storage_tx),
        }
    }

    pub(crate) fn get(&mut self, cx: isize, cy: isize) -> Option<&mut Chunk> {
        let chunk_cache_key = (cx, cy);
        let chunk_found = {
            // is it in the cache?
            self.chunks.peek(&chunk_cache_key).is_some()
        };
        // didn't find the chunk.
        // try to load it and insert it.
        if !chunk_found {
            let chunk_guid_bytes = create_seed_from_guid_x_y(self.owning_layer_guid, cx, cy);
            let chunk_unique_id = get_chunk_unique_id(chunk_guid_bytes, cx, cy, true);

            let chunk_storage_thread_handler = self.chunk_storage_thread_handler.lock().unwrap();
            let chunk = chunk_storage_thread_handler.load_chunk(&chunk_unique_id);
            if chunk.is_some() {
                self.chunks.insert(chunk_cache_key, chunk.unwrap());
            }
        }

        // now try to pull the chunk out of the cache
        let chunk_opt = self.chunks.get(&chunk_cache_key);
        chunk_opt
    }

    pub(crate) fn insert(&mut self, cx: isize, cy: isize, chunk: Chunk) {
        if self.chunks.len() == self.lru_cache_size {
            // the cache is full.
            // pop the oldest
            if let Some(((_cx, _cy), oldest_chunk)) = self.chunks.pop_oldest() {
                // need to store this chunk
                // self.save_chunk_to_store(cx, cy, oldest_chunk);
                if let Some(storage_tx) = &self.storage_tx {
                    storage_tx.send(oldest_chunk).expect("chunk failed to send");
                }
            }
        }
        // insert the new chunk
        self.chunks.insert((cx, cy), chunk);
    }
}
