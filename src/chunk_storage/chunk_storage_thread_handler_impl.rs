use crate::chunk::Chunk;
use hiivelabs_storage_lib::prelude::{SqliteStorageContainer, StorageContainer, UniqueId};
use std::sync::{mpsc, Arc, Mutex};
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
        storage_rx: mpsc::Receiver<Chunk>,
    ) -> Arc<Mutex<ChunkStorageThreadHandler>> {
        let storage_file_name = &format!("{owning_manager_guid}.world");
        let storage = SqliteStorageContainer::new(storage_file_name, true)
            .expect("failed to create storage container");

        let storage_thread_handler = Arc::new(Mutex::new(Self {
            owning_manager_guid,
            storage,
            thread_join_handle: None,
        }));

        // clone to set up thread starting
        let temp_clone = storage_thread_handler.clone();
        let thread_handle = ChunkStorageThreadHandler::init_thread(temp_clone, storage_rx);

        // clone to set thread handle.
        let temp_clone = storage_thread_handler.clone();
        let mut self_lock = temp_clone.lock().unwrap();
        self_lock.thread_join_handle = Some(thread_handle);

        storage_thread_handler
    }

    fn init_thread(
        arced_self: Arc<Mutex<Self>>,
        storage_rx: mpsc::Receiver<Chunk>,
    ) -> JoinHandle<()> {
        let thread_handle = thread::spawn(move || {
            while let Ok(chunk) = storage_rx.recv() {
                // Save chunk to disk
                let self_lock = arced_self.lock().unwrap();
                self_lock.save_chunk(chunk);
            }
        });
        thread_handle
    }

    pub(crate) fn join(&mut self) {
        if let Some(thread_handle) = self.thread_join_handle.take() {
            match thread_handle.join() {
                Ok(_) => println!("Storage thread has finished."),
                Err(e) => println!("Failed to join storage thread: {e:?}"),
            }
        }
    }

    pub(crate) fn save_chunk(&self, chunk: Chunk) {
        // Implement disk write operations
        let chunk_unique_id = chunk.get_unique_id(true);

        let save_result = self.storage.save_data_to_package(chunk.clone(), true);
        match save_result {
            Ok(_) => {
                log::info!("Saved chunk [{chunk_unique_id}] to disk");
            }
            Err(err) => {
                log::warn!("Chunk [{chunk_unique_id}] not saved to disk [{err}]",)
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
