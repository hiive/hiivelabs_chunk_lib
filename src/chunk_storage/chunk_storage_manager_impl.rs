use schnellru::{ByLength, LruMap};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use hiivelabs_storage_lib::prelude::UniqueId;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::chunk::Chunk;
use crate::chunk_seed_utils::chunk_seed_utils_impl::{
    create_seed_from_guid_x_y, get_chunk_unique_id,
};
use crate::chunk_storage::chunk_storage_message_impl::ChunkStorageMessage;
use crate::chunk_storage::chunk_storage_thread_handler_impl::ChunkStorageThreadHandler;

// TODO! create tests for ChunkStorageManager
// TODO! consider how to put ChunkStorageManager at top level.
// TODO! need to be able to request chunks by (x, y, z), where z could be layer guid
// TODO! Add tests for get_chunk_coords_for_hash_index (in layer)

// DONE! Maintain list of chunks in storage, for quick checking without having to hit the disk
// TODO! Make sure the above is thread-safe

const MAX_ATTEMPTS: usize = 5;

pub(crate) struct ChunkStorageManager {
    chunks: LruMap<(isize, isize), Chunk>,
    lru_cache_size: usize,
    owning_manager_guid: Uuid,
    owning_layer_guid: Uuid,
    chunk_storage_thread_handler: Arc<Mutex<ChunkStorageThreadHandler>>,
    stored_chunk_ids: HashSet<String>,
    storage_tx: Option<Sender<ChunkStorageMessage>>,
    shutdown_complete_rx: Receiver<bool>,
}

impl Drop for ChunkStorageManager {
    fn drop(&mut self) {
        // Flush existing chunks to disk
        let mut flush_count = 0;
        let to_flush_count = self.chunks.len();
        if let Some(storage_tx) = self.storage_tx.take() {
            for (chunk_cache_key, chunk) in self.chunks.drain() {
                let chunk_id = chunk.get_unique_id(true);
                match storage_tx.send(ChunkStorageMessage::ToStore(chunk)) {
                    Ok(_) => {
                        log::info!("Flush chunk dispatch [{chunk_id}] {chunk_cache_key:?} success");
                    }
                    Err(e) => {
                        log::error!(
                            "Flushing chunk: [{chunk_id}] {chunk_cache_key:?} failed- [{e:?}]"
                        )
                    }
                }
                flush_count += 1;
            }

            match storage_tx.send(ChunkStorageMessage::ShutDown) {
                Ok(_) => {
                    log::info!("ShutDown dispatched");
                }
                Err(e) => {
                    log::error!("ShutDown dispatch failure - [{e:?}]")
                }
            }
        }

        // wait for shutdown complete notification message
        match Some(&self.shutdown_complete_rx) {
            Some(rx) => match rx.recv() {
                Ok(shutdown_complete) => {
                    log::info!("ShutDown complete acknowledged: [{shutdown_complete}]");
                }
                Err(err) => {
                    log::error!("ShutDown complete acknowledgement error: [{err:?}]")
                }
            },
            None => {
                log::error!("No shutdown complete receiver!")
            }
        }
        // wait before attempting to shut stuff down.
        thread::sleep(std::time::Duration::from_millis(100));
        let mut chunk_storage_thread_handler = self.chunk_storage_thread_handler.lock().unwrap();
        log::info!("waiting for storage thread shutdown");
        chunk_storage_thread_handler.join();
        log::info!("flushed {flush_count}/{to_flush_count} chunks");
        log::info!("storage thread shutdown successfully");
    }
}

impl ChunkStorageManager {
    pub(crate) fn new(
        lru_cache_size: u32,
        owning_manager_guid_bytes: [u8; 16],
        owning_layer_guid_bytes: [u8; 16],
    ) -> Self {
        let chunks = {
            let mut hashmap = LruMap::new(ByLength::new(lru_cache_size + 1)); // + 1 for luck.
            hashmap.reserve_or_panic(lru_cache_size as usize + 1); // + 1 for luck.
            hashmap
        };
        let (storage_tx, storage_rx) = mpsc::channel();
        let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel();
        let owning_manager_guid = Uuid::from_bytes(owning_manager_guid_bytes);
        let (chunk_storage_thread_handler, stored_chunk_ids) =
            ChunkStorageThreadHandler::start(owning_manager_guid, storage_rx, shutdown_complete_tx);

        let owning_layer_guid = Uuid::from_bytes(owning_layer_guid_bytes);
        Self {
            chunks,
            lru_cache_size: lru_cache_size as usize,
            owning_manager_guid,
            owning_layer_guid,
            chunk_storage_thread_handler,
            stored_chunk_ids,
            storage_tx: Some(storage_tx),
            shutdown_complete_rx,
        }
    }

    ///
    /// peek_transient grabs a copy of the chunk without altering the lru.
    /// If it's in the lru, it peeks it out. If it's not, it loads it from disk.
    pub(crate) fn peek_transient(&self, cx: isize, cy: isize) -> Option<Chunk> {
        let chunk_cache_key = (cx, cy);
        let chunk_opt = self.chunks.peek(&chunk_cache_key).cloned();
        match chunk_opt {
            Some(_) => chunk_opt,
            None => {
                // is it in the storage cache?
                self.load_chunk_from_storage(cx, cy)
            }
        }
    }

    pub(crate) fn get(&mut self, cx: isize, cy: isize) -> Option<&mut Chunk> {
        let chunk_cache_key = (cx, cy);
        // log::info!("ChunkStorageManager:get([{chunk_cache_key:?}]) : START");
        let chunk_found = {
            // is it in the cache?
            self.chunks.peek(&chunk_cache_key).is_some()
        };
        // didn't find the chunk.
        // try to load it and insert it.
        if !chunk_found {
            let chunk = self.load_chunk_from_storage(cx, cy);

            if chunk.is_some() {
                // self.check_cache_size();
                self.insert(cx, cy, chunk.unwrap());
                // log::info!("ChunkStorageManager:get([{chunk_cache_key:?}]) : CHUNK LOADED");
            }
        }
        // else {
        //     log::info!("ChunkStorageManager:get([{chunk_cache_key:?}]) : found in mem-cache");
        // }

        // now try to pull the chunk out of the cache
        let chunk_opt = self.chunks.get(&chunk_cache_key);
        // match chunk_opt {
        //     None => {
        //         log::warn!("ChunkStorageManager:get([{chunk_cache_key:?}]) : DOESN'T exist");
        //     }
        //     Some(_) => {
        //         log::info!("ChunkStorageManager:get([{chunk_cache_key:?}]) : exists");
        //     }
        // }
        //log::info!("ChunkStorageManager:get([{chunk_cache_key:?}]) : END");
        chunk_opt
    }

    fn load_chunk_from_storage(&self, cx: isize, cy: isize) -> Option<Chunk> {
        // log::info!("ChunkStorageManager:get([{chunk_cache_key:?}]) : NOT FOUND in mem-cache");
        let chunk_guid_bytes = create_seed_from_guid_x_y(self.owning_layer_guid, cx, cy);
        let chunk_unique_id = get_chunk_unique_id(&chunk_guid_bytes, cx, cy, true);
        let chunk_cache_key = (cx, cy);
        // short circuit - don't load chunk if we know it's not in the cache
        if !self.stored_chunk_ids.contains(&chunk_unique_id) {
            log::info!("get() chunk not in storage: [{chunk_unique_id}] {chunk_cache_key:?}");
            return None;
        }

        log::info!("get() attempting load chunk: [{chunk_unique_id}] {chunk_cache_key:?}");

        // check if the chunk is in storage
        let chunk: Option<Chunk> = {
            let mut attempts = 0;
            loop {
                let chunk_storage_thread_handler_opt = self.chunk_storage_thread_handler.try_lock();
                let loaded_chunk = match chunk_storage_thread_handler_opt {
                    Ok(chunk_storage_thread_handler) => {
                        chunk_storage_thread_handler.load_chunk(&chunk_unique_id)
                    }
                    Err(err) => {
                        if attempts >= MAX_ATTEMPTS {
                            log::error!("failed to obtain read lock for [{chunk_unique_id}] {chunk_cache_key:?} : [{err:?}]");
                        }
                        attempts += 1;
                        None
                    }
                };

                // exit the loop if we've got something, or we're out of attempts
                if loaded_chunk.is_some() || attempts >= MAX_ATTEMPTS {
                    break loaded_chunk;
                }
                thread::sleep(std::time::Duration::from_millis(10));
            }
        };
        chunk
    }

    fn check_cache_size(&mut self) {
        while self.chunks.len() >= self.lru_cache_size {
            // the cache is full.
            // pop the oldest
            if let Some((chunk_cache_key, oldest_chunk)) = self.chunks.pop_oldest() {
                let chunk_unique_id = oldest_chunk.get_unique_id(true);

                if !oldest_chunk.is_dirty {
                    // no need to store it if it hasn't changed.
                    log::info!(
                        "insert() evicting clean chunk: [{chunk_unique_id}] {chunk_cache_key:?}"
                    );
                    continue;
                }

                log::info!(
                    "insert() evicting dirty chunk: [{chunk_unique_id}] {chunk_cache_key:?}"
                );
                // need to store this chunk
                if let Some(storage_tx) = &self.storage_tx {
                    storage_tx
                        .send(ChunkStorageMessage::ToStore(oldest_chunk))
                        .expect("chunk failed to send");
                }
            }
        }
    }

    pub(crate) fn insert(&mut self, cx: isize, cy: isize, chunk: Chunk) {
        log::info!(
            "insert() inserting chunk: [{}] ({cx}, {cy})",
            chunk.get_unique_id(true)
        );

        self.check_cache_size();

        // insert the new chunk
        self.chunks.insert((cx, cy), chunk);
    }
}
