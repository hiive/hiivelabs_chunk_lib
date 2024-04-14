use crate::chunk::Chunk;
use crate::chunk_storage::chunk_storage_message_impl::ChunkStorageMessage;
use hiivelabs_storage_lib::prelude::{SqliteStorageContainer, StorageContainer, UniqueId};
use log;
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use uuid::Uuid;

pub(crate) struct ChunkStorageThreadHandler {
    owning_manager_guid: Uuid,
    storage: SqliteStorageContainer,
    thread_join_handle: Option<JoinHandle<()>>,
}

impl ChunkStorageThreadHandler {
    pub(crate) fn start(
        owning_manager_guid: Uuid,
        storage_rx: Receiver<ChunkStorageMessage>,
        shutdown_complete_tx: Sender<bool>,
    ) -> (Arc<RwLock<ChunkStorageThreadHandler>>, HashSet<String>) {
        let storage_file_name = &format!("{owning_manager_guid}.world");
        let storage = SqliteStorageContainer::new(storage_file_name, true)
            .expect("failed to create storage container");

        let initial_package_contents = match storage.get_package_contents::<Chunk>() {
            Ok(contents) => HashSet::from_iter(contents),
            Err(_) => HashSet::<String>::new(),
        };

        let storage_thread_handler = Arc::new(RwLock::new(Self {
            owning_manager_guid,
            storage,
            thread_join_handle: None,
        }));

        // clone to set up thread starting
        let temp_clone = storage_thread_handler.clone();
        let thread_handle =
            ChunkStorageThreadHandler::init_thread(temp_clone, storage_rx, shutdown_complete_tx);

        // clone to set thread handle.
        let temp_clone = storage_thread_handler.clone();
        let self_lock_opt = temp_clone.try_write();
        match self_lock_opt {
            Ok(mut self_lock) => {
                self_lock.thread_join_handle = Some(thread_handle);
            }
            Err(err) => {
                log::error!("Error obtaining lock: [{err:?}]")
            }
        }

        (storage_thread_handler, initial_package_contents)
    }

    fn init_thread(
        arced_self: Arc<RwLock<Self>>,
        storage_rx: Receiver<ChunkStorageMessage>,
        shutdown_complete_tx: Sender<bool>,
    ) -> JoinHandle<()> {
        let thread_handle = thread::spawn(move || {
            while let Ok(storage_message) = storage_rx.recv() {
                // Save chunk to disk
                match storage_message {
                    ChunkStorageMessage::ToStore(mut chunk) => {
                        let chunk_cache_key = (
                            chunk.bounds.x / chunk.bounds.width as isize,
                            chunk.bounds.y / chunk.bounds.height as isize,
                        );
                        let chunk_unique_id = chunk.get_unique_id(true);
                        let mut attempts = 0;
                        loop {
                            let self_lock_opt = arced_self.try_write();
                            match self_lock_opt {
                                Ok(self_lock) => {
                                    self_lock.save_chunk(&mut chunk);
                                    break;
                                }
                                Err(err) => {
                                    attempts += 1;
                                    thread::sleep(std::time::Duration::from_millis(100));
                                    if attempts > 5 {
                                        log::error!("ChunkStorageThreadHandler: failed to obtain write lock for [{chunk_unique_id}] {chunk_cache_key:?} : [{err:?}]");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    ChunkStorageMessage::ShutDown => {
                        log::info!("ChunkStorageThreadHandler: Received shutdown request");
                        match shutdown_complete_tx.send(true) {
                            Ok(_) => {
                                log::info!(
                                    "ChunkStorageThreadHandler: Acknowledged shutdown request"
                                )
                            }
                            Err(err) => {
                                log::error!("ChunkStorageThreadHandler: Shutdown complete notification FAILED: [{err:?}]")
                            }
                        }
                        break;
                    }
                }
            }
            log::info!("ChunkStorageThreadHandler: Shutting Down...");
        });
        thread_handle
    }

    pub(crate) fn join(&mut self) {
        if let Some(thread_handle) = self.thread_join_handle.take() {
            match thread_handle.join() {
                Ok(_) => log::info!("Storage thread has finished."),
                Err(e) => log::info!("Failed to join storage thread: {e:?}"),
            }
        }
    }

    pub(crate) fn save_chunk(&self, chunk: &mut Chunk) {
        // Implement disk write operations
        let chunk_unique_id = chunk.get_unique_id(true);
        let chunk_coords = (
            chunk.bounds.x / chunk.bounds.width as isize,
            chunk.bounds.y / chunk.bounds.height as isize,
        );
        // we're saving it, so let's clear the dirty flag.
        chunk.is_dirty = false;
        let save_result = self.storage.save_data_to_package(chunk.clone(), true);
        match save_result {
            Ok(_) => {
                log::info!("Saved chunk [{chunk_unique_id}]:{chunk_coords:?} to disk");
            }
            Err(err) => {
                log::warn!("Chunk [{chunk_unique_id}]:{chunk_coords:?} not saved to disk [{err}]",)
            }
        }
    }

    pub(crate) fn load_chunk(&self, chunk_unique_id: &str) -> Option<Chunk> {
        let load_result = self
            .storage
            .load_data_from_package::<Chunk>(chunk_unique_id);
        match load_result {
            Ok(chunk) => {
                log::info!("Loaded chunk [{chunk_unique_id}] from disk",);
                Some(chunk)
            }
            Err(err) => {
                log::warn!("Chunk [{chunk_unique_id}] not loaded from disk [{err}]",);
                None
            }
        }
    }
}
