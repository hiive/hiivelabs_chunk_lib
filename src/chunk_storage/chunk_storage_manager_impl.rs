use schnellru::{ByLength, LruMap};
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
                        log::info!("ChunkStorageMessage: Flush chunk dispatch [{chunk_id}] {chunk_cache_key:?} success");
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
                    log::info!("ChunkStorageMessage::ShutDown dispatched");
                }
                Err(e) => {
                    log::error!("ChunkStorageMessage::ShutDown dispatch failure - [{e:?}]")
                }
            }
        }

        // wait for shutdown complete notification message
        match Some(&self.shutdown_complete_rx) {
            Some(rx) => {
                match rx.recv() {
                    Ok(shutdown_complete) => {
                        log::info!("ChunkStorageMessage::ShutDown complete acknowledged: [{shutdown_complete}]");
                    }
                    Err(err) => {
                        log::error!("ChunkStorageMessage::ShutDown complete acknowledgement error: [{err:?}]")
                    }
                }
            }
            None => {
                log::error!("No shutdown complete receiver!")
            }
        }
        // wait before attempting to shut stuff down.
        thread::sleep(std::time::Duration::from_millis(100));
        let mut chunk_storage_thread_handler = self.chunk_storage_thread_handler.lock().unwrap();
        log::info!("ChunkStorageManager: waiting for storage thread shutdown");
        chunk_storage_thread_handler.join();
        log::info!("ChunkStorageMessage: Flushed {flush_count}/{to_flush_count} chunks");
        log::info!("ChunkStorageManager: storage thread shutdown successfully");
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
        let chunk_storage_thread_handler =
            ChunkStorageThreadHandler::start(owning_manager_guid, storage_rx, shutdown_complete_tx);

        let owning_layer_guid = Uuid::from_bytes(owning_layer_guid_bytes);
        Self {
            chunks,
            lru_cache_size: lru_cache_size as usize,
            owning_manager_guid,
            owning_layer_guid,
            chunk_storage_thread_handler,
            storage_tx: Some(storage_tx),
            shutdown_complete_rx,
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
            // log::info!("ChunkStorageManager:get([{chunk_cache_key:?}]) : NOT FOUND in mem-cache");
            let chunk_guid_bytes = create_seed_from_guid_x_y(self.owning_layer_guid, cx, cy);
            let chunk_unique_id = get_chunk_unique_id(chunk_guid_bytes, cx, cy, true);

            log::info!("ChunkStorageManager:get() attempting load chunk: [{chunk_unique_id}] {chunk_cache_key:?}");

            // let chunk = {
            //     let chunk_storage_thread_handler =
            //         self.chunk_storage_thread_handler.lock().unwrap();
            //     chunk_storage_thread_handler.load_chunk(&chunk_unique_id)
            // };
            let chunk:Option<Chunk> = {
                let mut attempts = 0;
                loop {
                    let chunk_storage_thread_handler_opt = self.chunk_storage_thread_handler.try_lock();
                    let loaded_chunk = match chunk_storage_thread_handler_opt {
                        Ok(chunk_storage_thread_handler) => {
                            chunk_storage_thread_handler.load_chunk(&chunk_unique_id)
                        }
                        Err(err) => {
                            if attempts > 5 {
                                log::error!("failed to obtain read lock for [{chunk_unique_id}] {chunk_cache_key:?} : [{err:?}]");
                            }
                            None
                        }
                    };


                    attempts += 1;
                    thread::sleep(std::time::Duration::from_millis(10));
                    break loaded_chunk;
                }
            };

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

    fn check_cache_size(&mut self) {
        while self.chunks.len() >= self.lru_cache_size {
            // the cache is full.
            // pop the oldest
            if let Some((chunk_cache_key, oldest_chunk)) = self.chunks.pop_oldest() {
                let chunk_unique_id = oldest_chunk.get_unique_id(true);

                if !oldest_chunk.is_dirty {
                    // no need to store it if it hasn't changed.
                    log::info!("ChunkStorageManager:insert() evicting clean chunk: [{chunk_unique_id}] {chunk_cache_key:?}");
                    continue;
                }

                log::info!("ChunkStorageManager:insert() evicting dirty chunk: [{chunk_unique_id}] {chunk_cache_key:?}");
                // need to store this chunk

                if let Some(storage_tx) = &self.storage_tx {
                    // storage_tx.send(ChunkStorageMessage::ToStore((move | c| c)(oldest_chunk))).expect("chunk failed to send");
                    storage_tx
                        .send(ChunkStorageMessage::ToStore(oldest_chunk))
                        .expect("chunk failed to send");
                }
            }
        }
    }

    pub(crate) fn insert(&mut self, cx: isize, cy: isize, chunk: Chunk) {
        log::info!(
            "ChunkStorageManager:insert() inserting chunk: [{}] ({cx}, {cy})",
            chunk.get_unique_id(true)
        );

        self.check_cache_size();

        // insert the new chunk
        self.chunks.insert((cx, cy), chunk);
    }
}
