use hiivelabs_storage_lib::prelude::UniqueId;

#[cfg(feature = "multithreaded_chunk_generation")]
use indexmap::{IndexMap, IndexSet};
#[cfg(feature = "multithreaded_chunk_generation")]
use log::log;
use rustc_hash::FxHashMap;
#[cfg(feature = "multithreaded_chunk_generation")]
use rustc_hash::FxHasher;

use std::hash::BuildHasherDefault;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::chunk_layer::ChunkLayer;
use crate::chunk_seed_utils::chunk_seed_utils_impl::create_seed_from_guid_bytes_x_y;

use crate::tilemap_datasource::TileMapDataSource;

#[cfg(feature = "multithreaded_chunk_generation")]
use hiivelabs_rand_utils_lib::prelude::{
    create_worker_pool, shutdown_worker_pool, submit_message_to_worker_pool, WorkerPoolMessage,
};
use hiivelabs_rand_utils_lib::prelude::{IDim, TIndex, UDim};
#[cfg(feature = "multithreaded_chunk_generation")]
use std::any::Any;
#[cfg(feature = "multithreaded_chunk_generation")]
use std::sync::mpsc::{self, Receiver, RecvError};
#[cfg(feature = "multithreaded_chunk_generation")]
use std::thread;
#[cfg(feature = "multithreaded_chunk_generation")]
use std::thread::JoinHandle;

/// Manages a chunked 2D tilemap that automatically procedurally generates
/// additional procedural detail.
pub struct ChunkManager<T> {
    pub(crate) layers: Vec<Arc<RwLock<Option<ChunkLayer>>>>,
    /// The width in tiles of the top level map data.
    pub width: UDim,
    /// The height in tiles of the top level map data.
    pub height: UDim,
    pub(crate) owned_values: Vec<T>,
    pub(crate) out_of_bounds_value_index: TIndex,
    pub(crate) manager_guid_bytes: [u8; 16],
    #[cfg(feature = "multithreaded_chunk_generation")]
    thread_join_handle: Option<JoinHandle<()>>,
    #[cfg(feature = "multithreaded_chunk_generation")]
    worker_pool_name: String,
    #[cfg(feature = "multithreaded_chunk_generation")]
    job_complete_rx: Receiver<usize>,
}

#[cfg(feature = "multithreaded_chunk_generation")]
impl<T> Drop for ChunkManager<T> {
    fn drop(&mut self) {
        shutdown_worker_pool(&self.worker_pool_name);

        if let Some(thread_handle) = self.thread_join_handle.take() {
            match thread_handle.join() {
                Ok(_) => {
                    log::info!("Chunk Manager thread has shutdown.");
                    println!("Chunk Manager thread has shutdown.");
                }
                Err(e) => log::info!("Failed to join Chunk Manager thread: {e:?}"),
            }
        }
    }
}

impl<T: std::fmt::Debug> ChunkManager<T> {
    /// Initializes a new instance of the `ChunkManager<T>`.
    ///
    /// # Arguments
    ///
    /// * `source`: The source data for the top level map, implementing the [`TileMapDataSource<T>`] trait.
    /// * `layer_count`: The number of detail layers to procedurally generate.
    ///                  Each layer is four times the area of the previous layer.
    ///                  The `source` data is copied into layer `0`.
    /// * `layer_chunk_cache_size`: The number of procedurally generated chunks to keep in the cache.
    ///                  This is per layer. The recommended minimum value for most use case is `32`.
    /// * `chunk_width_in_tiles`: The width in tiles of the procedurally generated chunks.
    ///                  Each layer other than `0` is split into chunks.
    ///                  This must be exactly divisible by the `source` width.
    /// * `chunk_height_in_tiles`: The height in tiles of the procedurally generated chunks.
    ///                  Each layer other than `0` is split into chunks.
    ///                  This must be exactly divisible by the `source` height.
    /// * `chunk_padding_in_tiles`: The number of tiles by which to overlap chunks.
    ///                  The best value to use is `1`, as it minimizes edge artifacts
    ///                  with the current procedural generation method.
    ///
    /// * `guid`: The guid for this chunk manager. If none is provided, one will be generated.
    ///           This is used as an identifier for serialization/deserialization.
    ///
    /// returns: [`ChunkManager<T>`] initialized with the `source` data.
    pub fn new(
        source: Box<dyn TileMapDataSource<T>>,
        layer_count: UDim,
        layer_chunk_cache_size: u32,
        chunk_width_in_tiles: UDim,
        chunk_height_in_tiles: UDim,
        chunk_padding_in_tiles: UDim,
        guid: Option<Uuid>,
    ) -> Self {
        let width = source.width();
        let height = source.height();
        assert!(
            width > 0 && height > 0,
            "width and height must both be greater than zero"
        );
        assert!(
            chunk_width_in_tiles > 0
                && chunk_height_in_tiles > 0
                && chunk_width_in_tiles % 2 == 0
                && chunk_height_in_tiles % 2 == 0,
            "chunk_width and chunk_height must both be greater than zero, and even"
        );
        assert!(
            width % chunk_width_in_tiles == 0 && height % chunk_height_in_tiles == 0,
            "width/height must be exactly divisible by chunk width/height"
        );

        assert!(layer_count > 0, "layer_count must be greater than zero");
        assert!(
            layer_chunk_cache_size > 0,
            "layer_chunk_cache_size must be greater than zero (Recommended > 32)"
        );

        let is_new = guid.is_none(); //todo - use this to determine whether to look in storage
        let _is_new = is_new; // temp

        let manager_guid_bytes = guid.unwrap_or(Uuid::new_v4()).as_bytes().to_owned();

        // TODO - check
        // let's calculate a reasonable starting capacity for the owned_values vector.
        // For the top layer, we can expect the entire capacity to be needed.
        // for each subsequent layer, we can expect that 0.25 * ratio of layer area to previous layer
        // is needed. As each subsequent layer has 4 times the area of the previous layer, this
        // is 0.25 * 4 = 1.
        // so the maximum amount to reserve will be width * height * layer_count,
        // meaning that (assuming no sharing) the owned_values vector will contain
        // width * height * layer_count instances of T.

        // let's start out with that, and reevaluate as necessary.
        let mut owned_values = Vec::<T>::with_capacity((width * height * layer_count) as usize);

        // #[cfg(debug_assertions)]
        log::info!("reserved space: {}", width * height * layer_count);

        // create the layers
        let (layers, out_of_bounds_value_index) = ChunkManager::init_layers(
            manager_guid_bytes,
            &mut owned_values,
            source,
            layer_count,
            layer_chunk_cache_size,
            chunk_width_in_tiles,
            chunk_height_in_tiles,
            chunk_padding_in_tiles,
        );
        //#[cfg(multithreaded_chunk_generation)]
        #[cfg(feature = "multithreaded_chunk_generation")]
        let (thread_join_handle, worker_pool_name, job_complete_rx) =
            ChunkManager::<T>::setup_chunk_creation_threads(&manager_guid_bytes, layers.clone());

        Self {
            layers,
            width,
            height,
            owned_values,
            out_of_bounds_value_index,
            manager_guid_bytes,
            #[cfg(feature = "multithreaded_chunk_generation")]
            thread_join_handle,
            #[cfg(feature = "multithreaded_chunk_generation")]
            worker_pool_name,
            #[cfg(feature = "multithreaded_chunk_generation")]
            job_complete_rx,
        }
    }

    #[cfg(feature = "multithreaded_chunk_generation")]
    fn setup_chunk_creation_threads(
        manager_guid_bytes: &[u8; 16],
        layers: Vec<Arc<RwLock<Option<ChunkLayer>>>>,
    ) -> (Option<JoinHandle<()>>, String, Receiver<usize>) {
        // initialize the worker thread pool
        let num_cores = usize::min(num_cpus::get(), 6);
        let pool_name = get_chunk_manager_unique_id::<T>(manager_guid_bytes, true);

        let (job_tx, job_rx) =
            mpsc::channel::<(Option<usize>, Option<usize>, Option<Box<dyn Any + Send>>)>();
        let (job_complete_tx, job_complete_rx) = mpsc::channel::<usize>();
        let thread_handle = thread::spawn(move || {
            let mut thread_count_remaining = num_cores;
            while let Ok((id, info, result_opt)) = job_rx.recv() {
                match result_opt {
                    None => {
                        log::info!("Shutdown ack received for worker pool thread: [{id:?}]");
                        thread_count_remaining -= 1;
                    }
                    Some(result) => {
                        // log::info!("Result received for worker pool thread: [{id:?}]:{info:?}");
                        if let Ok(index_map) = result.downcast::<IndexMap<
                            (isize, isize),
                            Option<TIndex>,
                            BuildHasherDefault<FxHasher>,
                        >>() {
                            // `index_map` is now a `Box<IndexMap<(isize, isize), Option<TIndex>, BuildHasherDefault<FxHasher>>>`
                            let layer_id = info.expect("No layer id!");
                            let layer_arc = layers.get(layer_id).expect("Invalid layer arc");
                            let mut index_map = *index_map;

                            let mut attempts = 0;
                            loop {
                                let mut layer_lock_result = layer_arc.try_write();
                                match layer_lock_result {
                                    Ok(mut layer_lock) => {
                                        let mut layer =
                                            layer_lock.as_mut().expect("Can't get layer");
                                        for ((tx, ty), t_opt) in index_map.drain(..) {
                                            match t_opt {
                                                None => {
                                                    panic!("Should not be empty!")
                                                }
                                                Some(t) => {
                                                    let layer_set_result = layer.set_at(tx, ty, t);
                                                    if layer_set_result.is_err() {
                                                        log::error!("Error setting layer value: ({tx}, {ty}, {layer_id}) -> {t} : {}", layer_set_result.unwrap_err())
                                                    }
                                                    // println!("[{}, {layer_id}]  ({tx},{ty},{layer_id}) -> {t}", id.unwrap());
                                                }
                                            }
                                        }
                                        let _ = job_complete_tx.send(id.expect("No job id!"));
                                        break;
                                    }
                                    Err(err) => {
                                        attempts += 1;
                                        thread::sleep(std::time::Duration::from_millis(10));
                                        if attempts > 5 {
                                            log::error!("failed to obtain write lock for layer [{layer_id} : [{err:?}]");
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if thread_count_remaining <= 0 {
                    break;
                }
            }
            log::info!("Shutting down chunk manager thread.");
            // println!("Shutting down chunk manager thread.");
        });

        create_worker_pool(&pool_name, num_cores, Some(job_tx));
        (Some(thread_handle), pool_name, job_complete_rx)
    }

    /// Returns the bounds (including padding) for the specified layer.
    ///
    /// # Arguments
    ///
    /// * `x`: The requested layer `z`.
    /// * `include_padding`: `true` to include padding.
    ///
    /// returns: [`Result`],
    /// structured as `Ok(x_min: isize, y_min: isize, x_max: isize, y_max: isize)` or `Err(&str)`.
    ///
    /// Use as follows:
    /// ```ignore
    /// use chunk_lib::prelude::ChunkManager;
    /// let chunk_manager: ChunkManager<u8> = [... initialize chunk manager here ...];
    /// // assume an initialized chunk manager with at least 4 layers
    /// let bounds_result = chunk_manager.get_bounds_for_layer(3, false);
    /// if let(Ok((x_min, x_max, y_min, y_max))) = bounds_result {
    ///     for y in y_min..y_max {
    ///         for x in x_min..x_max {
    ///             // do something here...
    ///         }
    ///     }
    /// }
    pub fn get_bounds_for_layer(
        &self,
        z: usize,
        include_padding: bool,
    ) -> Result<(IDim, IDim, IDim, IDim), &str> {
        let some_layer = self.layers.get(z);
        match some_layer {
            Some(layer_rc) => {
                let mut attempts = 0;
                loop {
                    let layer_lock_result = layer_rc.try_read();
                    match layer_lock_result {
                        Ok(layer_lock) => {
                            let layer = layer_lock.as_ref().expect("Can't get layer");
                            return Ok(layer.tile_bounds.get_bound_coords(include_padding));
                        }
                        Err(err) => {
                            attempts += 1;
                            #[cfg(feature = "multithreaded_chunk_generation")]
                            thread::sleep(std::time::Duration::from_millis(10));
                            if attempts > 5 {
                                log::error!("Couldn't lock layer: {err:?}");
                                return Err("Couldn't lock layer");
                            }
                        }
                    }
                }
            }
            _ => Err("Layer z coordinate out of bounds"),
        }
    }

    /// Retrieve the tile at coordinates `(x, y)` in layer `z`, where `z` is the
    /// zero-indexed layer number.
    /// If the specified tile does not exist in the layer, the chunk(s) containing that tile will
    /// be generated on demand. A generated chunk is guaranteed to be identical every time it is generated.
    ///
    /// Layer `0` has the `width` and `height` of the source data.
    /// Each subsequent layer is double the `width` and `height` of the source data.
    /// The `(x, y)` coordinates are relative to the requested layer `z`.
    ///
    /// # Arguments
    ///
    /// * `x`: The _x_ coordinate in the requested layer `z`.
    /// * `y`: The _y_ coordinate in the requested layer `z`.
    /// * `z`: The layer to query.
    ///
    /// returns: [`Result`], structured as  `Ok(&T)` if the specified coordinates are in
    /// bounds, else `Err(&str)`.
    pub fn get_at(&self, x: IDim, y: IDim, z: UDim) -> Result<&T, &str> {
        let some_layer = self.layers.get(z as usize);

        match some_layer {
            Some(layer_rc) => {
                self.ensure_layer_chunks_are_complete(x, y, z);
                // the preceding method borrows layers,
                // so has to apply_shader before the rest of the method.
                let mut attempts = 0;
                loop {
                    let layer_lock_result = layer_rc.try_write();
                    match layer_lock_result {
                        Ok(mut layer_lock) => {
                            let layer = layer_lock.as_mut().expect("Can't get layer");

                            // if we are out of bounds, we can short circuit and
                            // return the oob value from the top layer.
                            if x < 0
                                || y < 0
                                || x >= layer.tile_bounds.width as IDim
                                || y >= layer.tile_bounds.width as IDim
                            {
                                return Ok(&self.owned_values[self.out_of_bounds_value_index]);
                            }

                            match layer.get_at(x, y) {
                                Some(ix) => return Ok(&self.owned_values[ix]),
                                _ => return Err("(x, y) coordinates out of bounds"),
                            }
                        }
                        Err(err) => {
                            attempts += 1;
                            #[cfg(feature = "multithreaded_chunk_generation")]
                            thread::sleep(std::time::Duration::from_millis(10));
                            if attempts > 5 {
                                log::error!("Couldn't lock layer: {err:?}");
                                return Err("Couldn't lock layer");
                            }
                        }
                    }
                }
            }
            _ => Err("Layer z coordinate out of bounds"),
        }
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `x`:
    /// * `y`:
    /// * `z`:
    ///
    /// returns: ()
    ///
    pub(crate) fn ensure_layer_chunks_are_complete(&self, x: IDim, y: IDim, z: UDim) {
        if z == 0 {
            // nothing to do - the top layer is always complete.
            return;
        }
        // let's build a map of coordinates
        // for the corresponding tile coordinates in each layer 0 <= z
        // layer z - 1's coordinates are half of layer z.
        let coord_map: Vec<(IDim, IDim)> = (0..=z)
            .map(|l| {
                let ld = (z - l) as u32;
                let f = 2_isize.pow(ld) as IDim;
                (x / f, y / f)
            })
            .collect();

        // iterate through the layers, from 1 to z, ensuring that the specified layer chunk
        // is complete, so it can be used to calculate the next layer corresponding chunk.
        #[cfg(not(feature = "multithreaded_chunk_generation"))] // this is the non-multithreaded one
        for (layer_id, (tx, ty)) in coord_map.iter().enumerate().take(z as usize + 1).skip(1) {
            let mut attempts = 0;
            loop {
                let layer_rc = self.layers.get(layer_id).expect("Can't get layer.");
                let layer_lock_result = layer_rc.try_write();
                match layer_lock_result {
                    Ok(mut layer_lock) => {
                        let layer = layer_lock.as_mut().expect("Can't get layer");
                        layer.ensure_chunk_is_complete(*tx, *ty);
                        break;
                    }
                    Err(err) => {
                        attempts += 1;
                        #[cfg(feature = "multithreaded_chunk_generation")]
                        thread::sleep(std::time::Duration::from_millis(10));
                        if attempts > 5 {
                            log::error!("Couldn't lock layer: {err:?}");
                            break;
                        }
                    }
                }
            }
        }

        #[cfg(feature = "multithreaded_chunk_generation")]
        {
            for (layer_id, (tx, ty)) in coord_map.iter().enumerate().take(z + 1).skip(1) {
                let layer_rc = self.layers.get(layer_id).expect("Can't get layer.");
                let tasks_opt = {
                    let mut attempts = 0;
                    loop {
                        let layer_lock_result = layer_rc.try_write();
                        match layer_lock_result {
                            Ok(mut layer_lock) => {
                                let mut layer = layer_lock.as_mut().expect("Can't get layer");
                                // layer.ensure_chunk_is_complete(*tx, *ty);
                                break layer.get_ensure_chunk_is_complete_work(*tx, *ty);
                            }
                            Err(_) => {
                                attempts += 1;
                                thread::sleep(std::time::Duration::from_millis(10));
                                if attempts > 5 {
                                    break None;
                                }
                            }
                        }
                    }
                };

                // submit tasks if there are any.
                if let Some(mut tasks) = tasks_opt {
                    let mut submitted_tasks = HashSet::with_capacity(tasks.len());
                    // now send the tasks
                    for task in tasks.drain(..) {
                        match &task {
                            WorkerPoolMessage::Shutdown => {}
                            WorkerPoolMessage::WorkerTask(t) => {
                                submitted_tasks.insert(t.task_id.expect("No task id"));
                                // println!("Submitting: {}, {}", t.task_id.unwrap(), t.task_info.unwrap())
                            }
                        }

                        submit_message_to_worker_pool(&self.worker_pool_name, task);
                    }
                    // need to wait for tasks to complete
                    while submitted_tasks.len() != 0 {
                        match self.job_complete_rx.recv() {
                            Ok(task_id) => {
                                submitted_tasks.remove(&task_id);
                                // println!("Removing task [{task_id}]");
                            }
                            Err(err) => {
                                panic!("Error removing job: {err}")
                            }
                        }
                    }
                }
            }
        }
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `guid`:
    /// * `owned_values`:
    /// * `source`:
    /// * `layer_count`:
    /// * `layer_chunk_cache_size`:
    /// * `chunk_width_in_tiles`:
    /// * `chunk_height_in_tiles`:
    /// * `chunk_padding_in_tiles`:
    ///
    /// returns: (Vec<Rc<RefCell<Option<ChunkLayer>>, Global>, Global>, usize)
    ///
    #[allow(clippy::too_many_arguments)]
    fn init_layers(
        manager_guid_bytes: [u8; 16],
        owned_values: &mut Vec<T>,
        mut source: Box<dyn TileMapDataSource<T>>,
        layer_count: UDim,
        layer_chunk_cache_size: u32,
        chunk_width_in_tiles: UDim,
        chunk_height_in_tiles: UDim,
        chunk_padding_in_tiles: UDim,
    ) -> (Vec<Arc<RwLock<Option<ChunkLayer>>>>, TIndex) {
        let width_in_chunks = source.width() / chunk_width_in_tiles;
        let height_in_chunks = source.height() / chunk_height_in_tiles;
        let out_of_bounds_value_index = source.get_default_out_of_bounds_value_index() as TIndex;

        // create the layers
        let mut layers = Vec::with_capacity(layer_count as usize);
        let mut prev_layer = Arc::new(RwLock::new(None));
        for layer_id in 0..layer_count as UDim {
            // each layer is double the width/height of the previous one.
            let layer_guid_bytes =
                create_seed_from_guid_bytes_x_y(&manager_guid_bytes, layer_id as isize, 0);
            let f = 2_usize.pow(layer_id as u32) as UDim;

            let (
                layer_chunk_width,
                layer_chunk_height,
                layer_width_in_chunks,
                layer_height_in_chunks,
                layer_chunk_padding_in_tiles,
                layer_default_oob_value,
            ) = {
                // set up layer initialization parameters
                if layer_id == 0 {
                    (
                        // layer 0 is a special case.
                        // it only has one chunk, and it's the same size as
                        // the source map.
                        source.width(),
                        source.height(),
                        1,
                        1,
                        0,
                        Some(out_of_bounds_value_index),
                    )
                } else {
                    (
                        // layers 1 to layer_count are made up of chunks.
                        // each layer has 4 times the number of chunks as the previous layer.
                        // chunks are fixed dimensions.
                        f * width_in_chunks,
                        f * height_in_chunks,
                        chunk_width_in_tiles,
                        chunk_height_in_tiles,
                        chunk_padding_in_tiles,
                        None,
                    )
                }
            };

            // create the layer
            let mut current_layer = ChunkLayer::new(
                Arc::clone(&prev_layer),
                layer_id,
                manager_guid_bytes,
                layer_guid_bytes,
                layer_chunk_cache_size,
                layer_width_in_chunks,
                layer_height_in_chunks,
                layer_chunk_padding_in_tiles,
                layer_chunk_width,
                layer_chunk_height,
                layer_default_oob_value,
            );

            if layer_id == 0 {
                // if we're on the top layer, populate it with the source map data
                Self::populate_top_layer_from_source(owned_values, &mut source, &mut current_layer);
            }
            // set prev layer to current layer for the next iteration
            prev_layer = ChunkLayer::make_layer_rc(Some(current_layer));
            // prev layer actually contains the current layer at this point,
            // so add it to the layer vector
            layers.push(Arc::clone(&prev_layer));
        }
        // return the layer collection, and the index of the layer 0 OOB tile.
        (layers, out_of_bounds_value_index)
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `with_padding`:
    ///
    /// returns: ()
    ///
    pub(crate) fn log_all_layer_index_diagnostics(&self, with_padding: bool) {
        log::info!("ChunkManager [{}]", self.get_unique_id(true));

        for layer_rc in &self.layers {
            let mut attempts = 0;
            loop {
                let layer_lock_result = layer_rc.try_write();
                match layer_lock_result {
                    Ok(mut layer_lock) => {
                        let layer = layer_lock.as_mut().expect("Can't get layer");

                        // for debug printing, replace the old oob value with the layer 0 one
                        let old_oob = layer.out_of_bounds_value_index;
                        layer.out_of_bounds_value_index = Some(self.out_of_bounds_value_index);
                        layer.log_diagnostics(with_padding);
                        // restore the old value when we're done.
                        layer.out_of_bounds_value_index = old_oob;
                        break;
                    }
                    Err(err) => {
                        attempts += 1;
                        #[cfg(feature = "multithreaded_chunk_generation")]
                        thread::sleep(std::time::Duration::from_millis(10));
                        if attempts > 5 {
                            log::error!("Couldn't lock layer: {err:?}");
                            break;
                        }
                    }
                }
            }
        }
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `with_padding`:
    ///
    /// returns: ()
    ///
    pub(crate) fn log_all_layer_value_diagnostics(&self, with_padding: bool) {
        log::info!("ChunkManager: [{}]", self.get_unique_id(true));

        let layer_bounds = {
            let mut lbs = Vec::with_capacity(self.layers.len());
            for layer_rc in &self.layers {
                let mut attempts = 0;
                loop {
                    let layer_lock_result = layer_rc.try_read();
                    match layer_lock_result {
                        Ok(layer_lock) => {
                            let layer = layer_lock.as_ref().expect("Can't get layer");
                            let print_bounds = layer.tile_bounds.get_bound_coords(with_padding);
                            let cropped_bounds = layer.tile_bounds.get_bound_coords(false);
                            lbs.push((print_bounds, cropped_bounds, layer.get_unique_id(true)));
                            break;
                        }
                        Err(err) => {
                            attempts += 1;
                            #[cfg(feature = "multithreaded_chunk_generation")]
                            thread::sleep(std::time::Duration::from_millis(10));
                            if attempts > 5 {
                                log::error!("Couldn't lock layer: {err:?}");
                                break;
                            }
                        }
                    }
                }
            }
            lbs
        };
        for (layer_id, (print_bounds, _cropped_bounds, layer_guid)) in
            layer_bounds.iter().enumerate()
        {
            let (x_min, y_min, x_max, y_max) = *print_bounds;

            log::info!("");
            log::info!("Layer: [{layer_id}]:[{layer_guid}] - VALUES");
            log::info!("");

            for y in y_min..y_max {
                let mut row: String = String::new();
                for x in x_min..x_max {
                    let result = self.get_at(x, y, layer_id as UDim);
                    match result {
                        Err(_) => {
                            row.push_str("-- ");
                        }
                        Ok(t) => {
                            row.push_str(format!("{t:?} ").as_str());
                        }
                    }
                }
                log::info!("{row}");
            }
            log::info!("");
        }
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `owned_values`:
    /// * `source`:
    /// * `current_layer`:
    ///
    /// returns: ()
    ///
    fn populate_top_layer_from_source(
        owned_values: &mut Vec<T>,
        source: &mut Box<dyn TileMapDataSource<T>>,
        current_layer: &mut ChunkLayer,
    ) {
        let width = source.width();
        let height = source.height();

        // build the map of vec index to (x, y) for use when draining the source vector
        // let mut ix_map = HashMap::with_capacity(width * height);
        // let mut ix_map = HashMap::<usize, (isize, isize), nohash_hasher::BuildNoHashHasher<usize>>::with_capacity_and_hasher(width * height, nohash_hasher::BuildNoHashHasher::default());
        let mut ix_map = FxHashMap::with_capacity_and_hasher(
            (width * height) as usize,
            BuildHasherDefault::default(),
        );
        for y in 0..height {
            for x in 0..width {
                if let Some(ix) = source.get_index_of(x, y) {
                    // we have the index into the source data of coordinates (x, y)
                    ix_map.insert(ix, (x as IDim, y as IDim));
                }
            }
        }

        // populate top layer
        let mut map_data = { source.take_data() }; // take the map data
        let layer0 = current_layer;
        for (ix, item) in map_data.drain(..).enumerate() {
            let (x, y) = ix_map[&ix];
            // store the value and set the layer coords with the index value
            // for the stored value
            owned_values.push(item);
            {
                let t_index: TIndex = owned_values.len() - 1;
                let _ = layer0.set_at(x, y, t_index);
            }
        }
        // sanity check
        let mut layer0_chunks = layer0.chunks.try_lock().expect("Can't lock chunks");
        let layer0_chunk = layer0_chunks.get(0, 0).expect("No parent chunk found.");
        let incomplete_count = layer0_chunk.get_unset_tile_count();
        let total_count = width * height;
        log::info!("incomplete: {incomplete_count}/{total_count}");
        assert!(layer0_chunk.is_complete());
    }
}
