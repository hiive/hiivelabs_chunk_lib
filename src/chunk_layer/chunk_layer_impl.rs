use indexmap::IndexMap;
use rustc_hash::FxHasher;

use std::hash::BuildHasherDefault;
use std::sync::{Arc, RwLock, Mutex};
use hiivelabs_storage_lib::prelude::UniqueId;
use smallvec::SmallVec;
use uuid::Uuid;

use crate::bounds::Bounds;
use crate::chunk::Chunk;
use crate::chunk_generator::chunk_generator_trait::ChunkGenerator;
use crate::chunk_seed_utils::chunk_seed_utils_impl::create_seed_from_guid_bytes_x_y;
use crate::chunk_storage::chunk_storage_manager_impl::ChunkStorageManager;

#[cfg(feature = "multithreaded_chunk_generation")]
use std::any::Any;
#[cfg(feature = "multithreaded_chunk_generation")]
use hiivelabs_rand_utils_lib::prelude::{Task, WorkerPoolMessage, WorkerPoolMessage::WorkerTask};
use hiivelabs_rand_utils_lib::prelude::{IDim, TIndex, UDim};


pub struct ChunkLayer {
    pub(crate) layer_id: UDim,
    pub(crate) chunk_bounds: Bounds, // this is in chunk-coords
    pub(crate) tile_bounds: Bounds,  // this is in tile-coords
    pub(crate) chunk_width_in_tiles: UDim,
    pub(crate) chunk_height_in_tiles: UDim,
    pub(crate) chunks: Mutex<ChunkStorageManager>,
    pub(crate) parent_layer: Arc<RwLock<Option<ChunkLayer>>>,
    pub(crate) out_of_bounds_value_index: Option<TIndex>,
    pub(crate) manager_guid_bytes: [u8; 16],
    pub(crate) layer_guid_bytes: [u8; 16],
}

impl ChunkLayer {
    pub(crate) fn make_layer_rc(layer: Option<ChunkLayer>) -> Arc<RwLock<Option<ChunkLayer>>> {
        Arc::new(RwLock::new(layer))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parent_layer: Arc<RwLock<Option<ChunkLayer>>>,
        layer_id: UDim,
        manager_guid_bytes: [u8; 16],
        layer_guid_bytes: [u8; 16],
        layer_chunk_lru_cache_size: u32,
        width_in_chunks: UDim,
        height_in_chunks: UDim,
        chunk_padding_in_tiles: UDim,
        chunk_width_in_tiles: UDim,
        chunk_height_in_tiles: UDim,
        out_of_bounds_value_index: Option<TIndex>,
    ) -> Self {
        let width_in_tiles = width_in_chunks * chunk_width_in_tiles;
        let height_in_tiles = height_in_chunks * chunk_height_in_tiles;

        Self {
            layer_id,
            manager_guid_bytes,
            layer_guid_bytes,
            chunk_bounds: Bounds {
                x: 0,
                y: 0,
                width: width_in_chunks,
                height: height_in_chunks,
                padding: 0,
            },
            tile_bounds: Bounds {
                x: 0,
                y: 0,
                width: width_in_tiles,
                height: height_in_tiles,
                padding: chunk_padding_in_tiles,
            },
            chunk_width_in_tiles,
            chunk_height_in_tiles,
            chunks: Mutex::new(ChunkStorageManager::new(
                layer_chunk_lru_cache_size,
                manager_guid_bytes,
                layer_guid_bytes,
            )),
            parent_layer,
            out_of_bounds_value_index,
        }
    }

    pub(crate) fn get_hash_index_for_chunk_coords(&self, cx: IDim, cy: IDim) -> Option<IDim> {
        if cx < 0
            || cx >= self.chunk_bounds.width as IDim
            || cy < 0
            || cy >= self.chunk_bounds.height as IDim
        {
            return None;
        }

        let ix = cx + (cy * self.chunk_bounds.width as IDim);
        Some(ix)
    }

    pub(crate) fn get_chunk_coords_for_hash_index(
        &self,
        hash_index: IDim,
    ) -> Option<(IDim, IDim)> {
        // Check if the index is within the valid range
        let width = self.chunk_bounds.width as IDim;
        let height = self.chunk_bounds.height as IDim;
        if hash_index < 0 || hash_index >= width * height {
            return None;
        }

        let cx = hash_index % width;
        let cy = hash_index / width;

        Some((cx, cy))
    }

    pub(crate) fn is_chunk_border_coord(&self, tx: IDim, ty: IDim) -> (bool, bool) {
        (
            // has to be inside outer bounds, and also within padding between chunks
            tx < 0
                || (tx > 0
                    && tx <= self.tile_bounds.width as IDim
                    && tx % self.chunk_width_in_tiles as IDim == 0),
            ty < 0
                || (ty > 0
                    && ty <= self.tile_bounds.height as IDim
                    && ty % self.chunk_height_in_tiles as IDim == 0),
        )
    }

    pub(crate) fn tile_coords_to_chunk_coords(&self, tx: IDim, ty: IDim) -> (IDim, IDim) {
        (
            tx / self.chunk_width_in_tiles as IDim,
            ty / self.chunk_height_in_tiles as IDim,
        )
    }

    pub(crate) fn chunk_coords_to_tile_coords(&self, cx: IDim, cy: IDim) -> (IDim, IDim) {
        (
            cx * self.chunk_width_in_tiles as IDim,
            cy * self.chunk_height_in_tiles as IDim,
        )
    }

    pub(crate) fn get_chunk_indices_for_tile_coords(
        &self,
        tx: IDim,
        ty: IDim,
    ) -> SmallVec<(IDim, (IDim, IDim)), 4> {
        // there are two main possibilities here.
        // 1. It's a chunk boundary, so multiple chunks will be returned.
        // 2. It's within a chunk, so only one chunk will be returned
        let mut chunk_indices = SmallVec::with_capacity(4);
        let padding = self.tile_bounds.padding as IDim;
        let width = self.tile_bounds.width as IDim;
        let height = self.tile_bounds.width as IDim;

        // short circuit if well out of bounds
        if tx < -padding || tx >= width + padding || ty < -padding || ty >= height + padding {
            // completely out of bounds - no matching chunks
            return chunk_indices;
        }

        // figure out a chunk that these coords are in
        let (cx, cy) = self.tile_coords_to_chunk_coords(tx, ty);
        let chunk_ix = self.get_hash_index_for_chunk_coords(cx, cy);
        // add the chunk index if it's valid
        match chunk_ix {
            Some(ix) => {
                // chunk index valid
                chunk_indices.push((ix, (cx, cy)));
            }
            None => {
                // if the chunk index is not valid, then we need to
                // take this into account...
            }
        }

        // are they on a boundary?
        let (x_is_boundary, y_is_boundary) = self.is_chunk_border_coord(tx, ty);

        if !x_is_boundary && !y_is_boundary {
            // easiest case; these coords are in one chunk only.
            // we can do a short circuit return
            return chunk_indices;
        } else {
            // calculate the boundary of the indexed chunk
            let (c_tx, c_ty) = self.chunk_coords_to_tile_coords(cx, cy);
            let chunk_r = c_tx + self.chunk_width_in_tiles as IDim;
            let chunk_b = c_ty + self.chunk_height_in_tiles as IDim;
            // check if boundary is left or right, top or bottom
            let d_cx = {
                if tx == chunk_r {
                    1 /* right */
                } else {
                    -1 /* left */
                }
            };
            let d_cy = {
                if ty == chunk_b {
                    1 /* down */
                } else {
                    -1 /* up */
                }
            };
            // check left/right
            if x_is_boundary {
                // it's on the x boundary, so do we need the chunk to the right/left?
                self.add_boundary_chunk_if_in_bounds(&mut chunk_indices, cx + d_cx, cy);
            }
            // check up/down
            if y_is_boundary {
                // it's on the y boundary, so do we need the chunk to the top/bottom?
                self.add_boundary_chunk_if_in_bounds(&mut chunk_indices, cx, cy + d_cy);
            }
            // check corners
            if x_is_boundary && y_is_boundary {
                self.add_boundary_chunk_if_in_bounds(&mut chunk_indices, cx + d_cx, cy + d_cy);
            }
        }

        chunk_indices.sort();
        chunk_indices
    }

    fn add_boundary_chunk_if_in_bounds(
        &self,
        chunk_indices: &mut SmallVec<(IDim, (IDim, IDim)), 4>,
        cx: IDim,
        cy: IDim,
    ) {
        let chunk_ix = self.get_hash_index_for_chunk_coords(cx, cy);
        if let Some(ix) = chunk_ix {
            // check we are in bounds
            if self.chunk_bounds.is_index_in_bounds(ix) {
                chunk_indices.push((ix, (cx, cy)))
            }
        }
    }

    pub(crate) fn ensure_chunk_exists_at_tile_coords(&mut self, tx: IDim, ty: IDim) {
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        self.ensure_chunk_exists_by_indices(&chunk_ixs);
    }

    pub(crate) fn ensure_chunk_exists_by_indices(
        &self,
        chunk_ixs: &SmallVec<(IDim, (IDim, IDim)), 4>,
    ) {
        let mut chunks = self.chunks.try_lock().expect("Can't lock chunks");
        for (_chunk_ix, (cx, cy)) in chunk_ixs {
            match chunks.get(*cx, *cy) {
                Some(_chunk) => {
                    // already exists. No action required.
                    // log::info!("Chunk [{}] exists in cache", _chunk.get_unique_id(true));
                }
                None => {
                    // create a new chunk
                    let chunk_guid = Uuid::from_bytes(create_seed_from_guid_bytes_x_y(
                        &self.layer_guid_bytes,
                        *cx as isize,
                        *cy as isize,
                    ));
                    let (c_tx, c_ty) = self.chunk_coords_to_tile_coords(*cx, *cy);
                    let new_chunk = Chunk::new(
                        c_tx,
                        c_ty,
                        self.chunk_width_in_tiles,
                        self.chunk_height_in_tiles,
                        self.tile_bounds.padding,
                        chunk_guid,
                    );
                    log::info!(
                        "Chunk [{}]:({cx},{cy}) created",
                        new_chunk.get_unique_id(true)
                    );
                    chunks.insert(*cx, *cy, new_chunk);
                }
            }
        }
    }

    pub fn set_at(&mut self, tx: IDim, ty: IDim, value: TIndex) -> Result<(), &str> {
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        self.ensure_chunk_exists_by_indices(&chunk_ixs);

        let mut error_count = 0;
        let mut chunks = self.chunks.try_lock().expect("Can't lock chunks");
        // let chunks = chunks_lock.().expect("Can't get chunks");
        for (_chunk_ix, (cx, cy)) in chunk_ixs {
            let chunk = chunks.get(cx, cy).unwrap(); // we know the chunk exists.

            let result = chunk.set_at(tx, ty, value);
            if result.is_err() {
                error_count += 1;
            }
        }
        if error_count == 0 {
            Ok(())
        } else {
            Err("Layer coordinates out of bounds")
        }
    }

    pub fn get_at(&mut self, tx: IDim, ty: IDim) -> Option<TIndex> {
        self.get_at_internal(tx, ty, false)
    }

    pub fn get_at_or_default(&mut self, tx: IDim, ty: IDim) -> Option<TIndex> {
        let chunk_opt = self.get_at_internal(tx, ty, true);
        match chunk_opt {
            None => self.out_of_bounds_value_index,
            Some(_) => chunk_opt,
        }
    }

    #[cfg(experimental)]
    pub(crate) fn peek_at_or_default(&mut self, tx: IDim, ty: IDim) -> Option<TIndex> {
        // let chunk_opt = self.get_at_internal(tx, ty, true);
        // match chunk_opt {
        //     None => self.out_of_bounds_value_index,
        //     Some(_) => chunk_opt,
        // }
        let (cropped_x_min, cropped_y_min, cropped_x_max, cropped_y_max) =
            self.tile_bounds.get_bound_coords(true);
        let oob =
            tx < cropped_x_min || tx >= cropped_x_max || ty < cropped_y_min || ty >= cropped_y_max;
        if oob {
            // short circuit
            return self.out_of_bounds_value_index;
        }

        //
        match self.get_first_chunk_index_at_tile_coords(tx, ty) {
            Some(chunk_ix) => {
                let (cx, cy) = self
                    .get_chunk_coords_for_hash_index(chunk_ix)
                    .expect("Invalid chunk index!"); // should be always good
                let chunks = self.chunks.borrow();
                let chunk = chunks.peek_transient(cx, cy);

                match chunk {
                    Some(chunk) => chunk.get_at(tx, ty),
                    _ => None, // Either the index is out of bounds or the Option<ChunkTile<T>> is None
                }
            }
            None => None,
        }
    }

    #[inline(always)]
    fn get_at_internal(&mut self, tx: IDim, ty: IDim, include_padding: bool) -> Option<TIndex> {
        let (cropped_x_min, cropped_y_min, cropped_x_max, cropped_y_max) =
            self.tile_bounds.get_bound_coords(include_padding);
        let oob =
            tx < cropped_x_min || tx >= cropped_x_max || ty < cropped_y_min || ty >= cropped_y_max;
        if oob {
            // short circuit
            return self.out_of_bounds_value_index;
        }

        //
        let opt_chunk_ix = self.get_first_chunk_index_at_tile_coords(tx, ty);
        match opt_chunk_ix {
            Some(chunk_ix) => {
                let (cx, cy) = self
                    .get_chunk_coords_for_hash_index(chunk_ix)
                    .expect("Invalid chunk index!"); // should be always good

                let mut chunks = self.chunks.try_lock().expect("Can't lock chunks");
                let chunk = chunks.get(cx, cy);

                match chunk {
                    Some(chunk) => chunk.get_at(tx, ty),
                    _ => None, // Either the index is out of bounds or the Option<ChunkTile<T>> is None
                }
            }
            None => None,
        }
    }

    fn get_first_chunk_index_at_tile_coords(&mut self, tx: IDim, ty: IDim) -> Option<IDim> {
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        if chunk_ixs.is_empty() {
            log::warn!(
                "no chunks found for: ({tx}, {ty}), {} : (w:{}, h:{})",
                self.layer_id,
                self.tile_bounds.width,
                self.tile_bounds.height
            );
            return None;
        }
        self.ensure_chunk_exists_by_indices(&chunk_ixs);
        let (chunk_ix, _) = chunk_ixs[0];
        Some(chunk_ix)
    }

    #[inline(always)]
    pub(crate) fn convert_to_parent_layer_tile_coordinates(
        &self,
        tx: IDim,
        ty: IDim,
    ) -> (IDim, IDim) {
        // the source coordinates in the parent layer
        // will be half of the destination coordinates,
        // as the parent layer is half the size in each dimension
        (tx / 2, ty / 2)
    }

    #[cfg(feature = "multithreaded_chunk_generation")]
    pub(crate) fn get_ensure_chunk_is_complete_work(
        &mut self,
        tx: IDim,
        ty: IDim,
    ) -> Option<Vec<WorkerPoolMessage>> {
        if self.layer_id == 0 {
            // nothing to do.
            // layer zero is always considered complete as it's
            // initialized from the user-provided map data.
            return None;
        }

        // In order to ensure a chunk in this layer is complete,
        // the source chunks in the chain of parent layers also need to be complete.
        // note that there may be more than one source chunk in the parent layer,
        // depending on padding boundaries coinciding in the layer chain.

        // get the chunk indices fot the specified tile coordinates,
        // There may be more than one chunk to complete if (tx, ty) is within the padding
        // boundary.
        // and make sure the chunk(s) exists.
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        self.ensure_chunk_exists_by_indices(&chunk_ixs);

        let mut tasks = Vec::with_capacity(chunk_ixs.len());
        // iterate through the chunks.
        for (chunk_ix, (cx, cy)) in &chunk_ixs {
            // we know the chunk exists, because we ensured it earlier.
            let chunk_bounds_opt = {
                let mut chunks = self.chunks.try_lock().expect("Can't lock chunks");
                let chunk = chunks.get(*cx, *cy).expect("Chunk should be here");
                if chunk.is_complete() {
                    None
                } else {
                    Some(chunk.bounds.clone())
                }
            };
            if let Some(chunk_bounds) = chunk_bounds_opt {
                // the chunk has unset tiles, so let's complete it.
                // let's get the parent tiles that cover this chunk
                let parent_tiles = self.get_parent_tiles_for_expansion(&chunk_bounds);

                // get the child tiles that we are going to expand into
                let (this_layer_x0, this_layer_y0, this_layer_x1, this_layer_y1) =
                    chunk_bounds.get_bound_coords(true);
                let w = 2 + this_layer_x1 - this_layer_x0;
                let h = 2 + this_layer_y1 - this_layer_y0;
                let s = (w * h) as usize;

                let child_tiles = {
                    let mut child_tiles: IndexMap<
                        (IDim, IDim),
                        Option<TIndex>,
                        BuildHasherDefault<FxHasher>,
                    > = IndexMap::with_capacity_and_hasher(s, BuildHasherDefault::default());
                    for this_layer_y in this_layer_y0..this_layer_y1 {
                        for this_layer_x in this_layer_x0..this_layer_x1 {
                            let child_tile_value = self.get_at(this_layer_x, this_layer_y);
                            child_tiles.insert((this_layer_x, this_layer_y), child_tile_value);
                        }
                    }
                    child_tiles
                };

                // create the chunk work task
                // let task = WorkerPoolMessage::WorkerTask()
                // generate the chunk.
                let manager_guid_bytes = self.manager_guid_bytes.clone();
                let layer_guid_bytes = self.layer_guid_bytes.clone();
                let job: Box<dyn FnOnce() -> Option<Box<dyn Any + Send>> + Send> = Box::new(
                    move || {
                        let chunk_generator =
                        crate::chunk_generator::chunk_seeded_interpolator_impl::ChunkSeededInterpolator;
                        let result = chunk_generator.generate_chunk_work_from_parent(
                            parent_tiles,
                            manager_guid_bytes,
                            layer_guid_bytes,
                            chunk_bounds,
                            child_tiles,
                        );
                        // Assuming `result` can be turned into `dyn Any + Send`. You may need to adjust types or wrap further.
                        Some(Box::new(result) as Box<dyn Any + Send>)
                    },
                );
                let task_id = *chunk_ix as UDim;
                let task = WorkerPoolMessage::WorkerTask(Task {
                    priority: self.layer_id,
                    task_id: Some(task_id),
                    task_info: Some(self.layer_id),
                    job,
                });
                tasks.push(task);
                // chunk_generator.generate_chunk_from_parent(chunk_bounds, self, parent_tiles);
            }
        }
        Some(tasks)
    }

    pub(crate) fn ensure_chunk_is_complete(&mut self, tx: IDim, ty: IDim) {
        if self.layer_id == 0 {
            // nothing to do.
            // layer zero is always considered complete as it's
            // initialized from the user-provided map data.
            return;
        }

        // In order to ensure a chunk in this layer is complete,
        // the source chunks in the chain of parent layers also need to be complete.
        // note that there may be more than one source chunk in the parent layer,
        // depending on padding boundaries coinciding in the layer chain.

        // get the chunk indices fot the specified tile coordinates,
        // There may be more than one chunk to complete if (tx, ty) is within the padding
        // boundary.
        // and make sure the chunk(s) exists.
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        self.ensure_chunk_exists_by_indices(&chunk_ixs);

        // iterate through the chunks.
        let chunk_generator =
            crate::chunk_generator::chunk_seeded_interpolator_impl::ChunkSeededInterpolator;
        // let chunk_generator = crate::chunk_generator::chunk_doubler_impl::ChunkDoubler;

        for (_chunk_ix, (cx, cy)) in &chunk_ixs {
            // we know the chunk exists, because we ensured it earlier.
            let chunk_bounds_opt = {
                let mut chunks = self.chunks.try_lock().expect("Can't lock chunks");
                let chunk = chunks.get(*cx, *cy).expect("Chunk should be here");
                if chunk.is_complete() {
                    None
                } else {
                    Some(chunk.bounds.clone())
                }
            };
            if let Some(chunk_bounds) = chunk_bounds_opt {
                // the chunk has unset tiles, so let's complete it.
                // let's get the parent tiles that cover this chunk
                let parent_tiles = self.get_parent_tiles_for_expansion(&chunk_bounds);

                // generate the chunk.
                chunk_generator.generate_chunk_from_parent(chunk_bounds, self, parent_tiles);
            }
        }
    }

    fn get_parent_tiles_for_expansion(
        &mut self,
        chunk_bounds: &Bounds,
    ) -> IndexMap<(IDim, IDim), TIndex, BuildHasherDefault<FxHasher>> {
        // let's get the parent layer tiles that we are going to need...
        // a bit ugly, but it will work
        let parent_tiles = {
            let child_capacity = chunk_bounds.get_tile_count(true);
            let mut expansion_tiles =
                IndexMap::with_capacity_and_hasher(child_capacity, BuildHasherDefault::default());

            // get the parent layer.
            // it has to be mutable, because we are accessing chunks in an lru cache which
            // can change based on retrieval.
            // let mut parent_layer_ref = self.parent_layer.borrow_mut();
            if let Ok(mut layer_lock) = self.parent_layer.try_write() {
                let parent_layer = layer_lock.as_mut().unwrap();

                let (this_layer_x0, this_layer_y0, this_layer_x1, this_layer_y1) =
                    chunk_bounds.get_bound_coords(true);

                let parent_capacity = parent_layer.tile_bounds.get_tile_count(true);
                let mut parent_tiles_cache: IndexMap<
                    (IDim, IDim),
                    TIndex,
                    BuildHasherDefault<FxHasher>,
                > = IndexMap::with_capacity_and_hasher(
                    parent_capacity,
                    BuildHasherDefault::default(),
                );

                let padding = self.tile_bounds.padding as IDim;

                for this_layer_y in this_layer_y0 - padding..this_layer_y1 + padding {
                    for this_layer_x in this_layer_x0 - padding..this_layer_x1 + padding {
                        // the parent coordinates in the parent layer
                        let (parent_x, parent_y) = parent_layer
                            .convert_to_parent_layer_tile_coordinates(this_layer_x, this_layer_y);
                        // check to see if we cached it.
                        if let Some(tile_value) = parent_tiles_cache.get(&(parent_x, parent_y)) {
                            // short circuit if we did
                            expansion_tiles.insert((this_layer_x, this_layer_y), *tile_value);
                            continue;
                        }

                        // get the parent tile
                        let tile_value = {
                            match parent_layer.get_at_or_default(parent_x, parent_y) {
                                None => {
                                    // the source layer tile is unset
                                    // we need to call this method recursively
                                    // for the parent layer at the parent coordinates

                                    // we don't do this for multithreading
                                    #[cfg(not(feature = "multithreaded_chunk_generation"))]
                                    parent_layer.ensure_chunk_is_complete(parent_x, parent_y);

                                    // get the parent tile again. It should be set this time.

                                    parent_layer
                                        .get_at(parent_x, parent_y)
                                        .expect("Parent chunk tile is not set.")
                                }
                                Some(t) => t,
                            }
                        };
                        // save in expansion tiles
                        expansion_tiles.insert((this_layer_x, this_layer_y), tile_value);
                        // put in cache.
                        parent_tiles_cache.insert((parent_x, parent_y), tile_value);
                    }
                }
            }
            expansion_tiles
        };
        parent_tiles
    }

    pub(crate) fn log_diagnostics(&mut self, with_padding: bool) {
        let id = self.layer_id;
        let unique_id = self.get_unique_id(true);
        log::info!("");

        log::info!("Layer: [{id}]:[{unique_id}] - INDICES");
        log::info!("");

        let bb = (
            0,
            0,
            self.tile_bounds.width as IDim,
            self.tile_bounds.height as IDim,
        );
        log::trace!("(x, y, w, h) = {bb:?}");

        let (x_min, y_min, x_max, y_max) = self.tile_bounds.get_bound_coords(with_padding);
        log::trace!(
            "(x_min, y_min, x_max, y1) = {:?}",
            (x_min, y_min, x_max, y_max)
        );

        for y in y_min..y_max {
            let mut row: String = String::new();
            for x in x_min..x_max {
                let ov = self.get_at(x, y);
                let (xb, yb) = self.is_chunk_border_coord(x, y);
                match ov {
                    None => {
                        if xb || yb {
                            row.push_str(".. ");
                        } else {
                            row.push_str("-- ");
                        }
                    }
                    Some(v) => {
                        let v = (v % 256) as u8;

                        if xb || yb {
                            row.push_str(std::format!("{v:02x} ").as_str());
                        } else {
                            row.push_str(std::format!("{v:02X} ").as_str());
                        }
                    }
                }
            }
            log::info!("{row}");
        }
        log::info!("");
    }
}
