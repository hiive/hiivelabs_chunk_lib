use std::cell::RefCell;
use std::rc::Rc;

use hiivelabs_storage_lib::prelude::{SqliteStorageContainer, StorageContainer, UniqueId};
use schnellru::{ByLength, LruMap};
use smallvec::SmallVec;
use uuid::Uuid;

use crate::bounds::Bounds;
use crate::chunk::Chunk;
use crate::chunk_manager::create_seed_from_guid_bytes_x_y;

pub type TIndex = usize;

pub struct ChunkLayer {
    pub(crate) layer_id: usize,
    pub(crate) layer_chunk_lru_cache_size: u32,
    pub(crate) chunk_bounds: Bounds, // this is in chunk-coords
    pub(crate) tile_bounds: Bounds,  // this is in tile-coords
    pub(crate) chunk_width_in_tiles: usize,
    pub(crate) chunk_height_in_tiles: usize,
    pub(crate) chunks: RefCell<LruMap<isize, Chunk>>,
    pub(crate) parent_layer: Rc<RefCell<Option<ChunkLayer>>>,
    pub(crate) out_of_bounds_value_index: Option<TIndex>,
    pub(crate) guid_bytes: [u8; 16],
}

impl ChunkLayer {
    pub(crate) fn make_layer_rc(layer: Option<ChunkLayer>) -> Rc<RefCell<Option<ChunkLayer>>> {
        Rc::new(RefCell::new(layer))
    }

    pub fn new(
        parent_layer: Rc<RefCell<Option<ChunkLayer>>>,
        layer_id: usize,
        layer_guid_bytes: [u8; 16],
        layer_chunk_lru_cache_size: u32,
        width_in_chunks: usize,
        height_in_chunks: usize,
        chunk_padding_in_tiles: usize,
        chunk_width_in_tiles: usize,
        chunk_height_in_tiles: usize,
        out_of_bounds_value_index: Option<TIndex>,
    ) -> Self {
        let width_in_tiles = width_in_chunks * chunk_width_in_tiles;
        let height_in_tiles = height_in_chunks * chunk_height_in_tiles;
        let chunks = {
            let hashmap_capacity = width_in_chunks * height_in_chunks;
            let mut hashmap = LruMap::new(ByLength::new(layer_chunk_lru_cache_size));
            hashmap.reserve_or_panic(hashmap_capacity);
            RefCell::new(hashmap)
        };

        Self {
            layer_id,
            guid_bytes: layer_guid_bytes,
            layer_chunk_lru_cache_size,
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
            chunks,
            parent_layer,
            out_of_bounds_value_index,
        }
    }

    pub(crate) fn get_hash_index_for_chunk_coords(&self, cx: isize, cy: isize) -> Option<isize> {
        if cx < 0
            || cx >= self.chunk_bounds.width as isize
            || cy < 0
            || cy >= self.chunk_bounds.height as isize
        {
            return None;
        }

        let ix = cx + (cy * self.chunk_bounds.width as isize);
        Some(ix)
    }

    pub(crate) fn is_chunk_border_coord(&self, tx: isize, ty: isize) -> (bool, bool) {
        (
            // has to be inside outer bounds, and also within padding between chunks
            tx < 0
                || (tx > 0
                    && tx <= self.tile_bounds.width as isize
                    && tx % self.chunk_width_in_tiles as isize == 0),
            ty < 0
                || (ty > 0
                    && ty <= self.tile_bounds.height as isize
                    && ty % self.chunk_height_in_tiles as isize == 0),
        )
    }

    pub(crate) fn tile_coords_to_chunk_coords(&self, tx: isize, ty: isize) -> (isize, isize) {
        (
            tx / self.chunk_width_in_tiles as isize,
            ty / self.chunk_height_in_tiles as isize,
        )
    }

    pub(crate) fn chunk_coords_to_tile_coords(&self, cx: isize, cy: isize) -> (isize, isize) {
        (
            cx * self.chunk_width_in_tiles as isize,
            cy * self.chunk_height_in_tiles as isize,
        )
    }

    pub(crate) fn get_chunk_indices_for_tile_coords(
        &self,
        tx: isize,
        ty: isize,
    ) -> SmallVec<(isize, (isize, isize)), 4> {
        // there are two main possibilities here.
        // 1. It's a chunk boundary, so multiple chunks will be returned.
        // 2. It's within a chunk, so only one chunk will be returned
        let mut chunk_indices = SmallVec::with_capacity(4);
        let padding = self.tile_bounds.padding as isize;
        let width = self.tile_bounds.width as isize;
        let height = self.tile_bounds.width as isize;

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
            let chunk_r = c_tx + self.chunk_width_in_tiles as isize;
            let chunk_b = c_ty + self.chunk_height_in_tiles as isize;
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

        chunk_indices
    }

    fn add_boundary_chunk_if_in_bounds(
        &self,
        chunk_indices: &mut SmallVec<(isize, (isize, isize)), 4>,
        cx: isize,
        cy: isize,
    ) {
        let chunk_ix = self.get_hash_index_for_chunk_coords(cx, cy);
        if let Some(ix) = chunk_ix {
            // check we are in bounds
            if self.chunk_bounds.is_index_in_bounds(ix) {
                chunk_indices.push((ix, (cx, cy)))
            }
        }
    }

    pub(crate) fn ensure_chunk_exists_at_tile_coords(&mut self, tx: isize, ty: isize) {
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        self.ensure_chunk_exists_by_indices(&chunk_ixs);
    }

    pub(crate) fn ensure_chunk_exists_by_indices(
        &self,
        chunk_ixs: &SmallVec<(isize, (isize, isize)), 4>,
    ) {
        let mut chunks = self.chunks.borrow_mut();
        for (chunk_ix, (cx, cy)) in chunk_ixs {
            match chunks.get(chunk_ix) {
                Some(_) => {
                    // already exists. No action required.
                }
                None => {
                    // create a new chunk
                    let (c_tx, c_ty) = self.chunk_coords_to_tile_coords(*cx, *cy);
                    let guid = Uuid::from_bytes(create_seed_from_guid_bytes_x_y(
                        &self.guid_bytes,
                        c_tx,
                        c_ty,
                    ));
                    let new_chunk = Chunk::new(
                        c_tx,
                        c_ty,
                        self.chunk_width_in_tiles,
                        self.chunk_height_in_tiles,
                        self.tile_bounds.padding,
                        guid,
                    );
                    if chunks.len() == self.layer_chunk_lru_cache_size as usize {
                        // the cache is full.
                        // pop the oldest
                        if let Some((ix, oldest_chunk)) = chunks.pop_oldest() {
                            // need to store this chunk
                            self.store_chunk(ix, oldest_chunk);
                        }
                    }
                    // insert the new chunk
                    chunks.insert(*chunk_ix, new_chunk);
                }
            }
        }
    }

    pub(crate) fn store_chunk(&self, chunk_ix: isize, chunk: Chunk) {
        // todo - make sure we store sizes in the db
        // let encoded = bitcode::encode(&chunk);
        // let compressed = compress_to_vec(encoded.as_slice(), 6);
        // let _chunk_ix = chunk_ix; // temp
        // let _compressed = compressed;

        let storage = SqliteStorageContainer::new("test.db", true).unwrap();
        let result = storage.save_data_to_package(chunk, true).unwrap();
        let _test = storage.load_data_from_package::<Chunk>(result.as_str());
        let _chunk_ix = chunk_ix;
        println!("{result}");
        println!()
    }

    pub fn set_at(&mut self, tx: isize, ty: isize, value: TIndex) -> Result<(), &str> {
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        self.ensure_chunk_exists_by_indices(&chunk_ixs);

        let mut error_count = 0;
        for (chunk_ix, _) in chunk_ixs {
            let mut chunks = self.chunks.borrow_mut();
            let chunk = chunks.get(&chunk_ix).unwrap(); // we know the chunk exists.

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

    pub fn get_at(&mut self, tx: isize, ty: isize) -> Option<TIndex> {
        let (cropped_x_min, cropped_y_min, cropped_x_max, cropped_y_max) =
            self.tile_bounds.get_bound_coords(false);
        let oob =
            tx < cropped_x_min || tx >= cropped_x_max || ty < cropped_y_min || ty >= cropped_y_max;
        if oob {
            // short circuit
            return self.out_of_bounds_value_index;
        }

        //
        let opt_chunk_idx = self.get_first_chunk_index_at_tile_coords(tx, ty);
        match opt_chunk_idx {
            Some(chunk_idx) => {
                let mut chunks = self.chunks.borrow_mut();
                let chunk = chunks.get(&chunk_idx);

                match chunk {
                    Some(chunk) => chunk.get_at(tx, ty),
                    _ => None, // Either the index is out of bounds or the Option<ChunkTile<T>> is None
                }
            }
            None => None,
        }
    }

    pub fn get_at_or_default(&mut self, tx: isize, ty: isize) -> Option<TIndex> {
        let (min_x, min_y, max_x, max_y) = self.tile_bounds.get_bound_coords(true);
        if tx < min_x || tx >= max_x || ty < min_y || ty >= max_y {
            // short circuit
            return self.out_of_bounds_value_index;
        }
        let opt_chunk_idx = self.get_first_chunk_index_at_tile_coords(tx, ty);
        match opt_chunk_idx {
            Some(chunk_idx) => {
                let mut chunks = self.chunks.borrow_mut();
                let chunk = chunks.get(&chunk_idx);

                match chunk {
                    Some(chunk) => chunk.get_at_or_default(tx, ty, self.out_of_bounds_value_index),
                    _ => {
                        // println!("Got default value 2: {:?}", self.out_of_bounds_value_index);
                        self.out_of_bounds_value_index
                    }
                }
            }
            None => {
                // println!("Got default value 3: {:?}", self.out_of_bounds_value_index);
                self.out_of_bounds_value_index
            }
        }
    }

    fn get_first_chunk_index_at_tile_coords(&mut self, tx: isize, ty: isize) -> Option<isize> {
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        if chunk_ixs.is_empty() {
            log::warn!(
                "NO CHUNKS FOUND FOR: ({tx}, {ty}), {} : (w:{}, h:{})",
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

    pub(crate) fn convert_to_parent_layer_tile_coordinates(
        &self,
        tx: isize,
        ty: isize,
    ) -> (isize, isize) {
        // the source coordinates in the parent layer
        // will be half of the destination coordinates,
        // as the parent layer is half the size in each dimension
        (tx / 2, ty / 2)
    }

    pub(crate) fn ensure_chunk_is_complete(&self, tx: isize, ty: isize) {
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
        let mut chunks = self.chunks.borrow_mut();
        // println!();
        for (chunk_ix, _) in &chunk_ixs {
            // we know the chunk exists, because we ensured it earlier.
            let chunk = chunks.get(chunk_ix).unwrap();
            if !chunk.is_complete() {
                // the chunk has unset tiles, so let's complete it.
                // todo: see if it's in the disk cache

                // for initial purposes, we are just going to do a simple doubling up
                // of the parent.

                // get the parent layer.
                // it has to be mutable, because we are accessing chunks in an lru cache which
                // can change based on retrieval.
                let mut parent_layer_ref = self.parent_layer.borrow_mut();
                let parent_layer: &mut ChunkLayer = parent_layer_ref
                    .as_mut()
                    .expect("No parent layer for this layer");

                // get the layer relative tile coordinates for the area that needs
                // to be set in this chunk
                let (this_layer_x0, this_layer_y0, this_layer_x1, this_layer_y1) =
                    chunk.bounds.get_bound_coords(true);

                // loop over the layer tile coordinates
                for this_layer_y in this_layer_y0..this_layer_y1 {
                    for this_layer_x in this_layer_x0..this_layer_x1 {
                        // the parent coordinates in the parent layer
                        let (parent_x, parent_y) = self
                            .convert_to_parent_layer_tile_coordinates(this_layer_x, this_layer_y);

                        // get the parent tile
                        let parent_tile = {
                            // check if the parent tile is set.
                            // (nb. only the top layer has a default tile value set).
                            match parent_layer.get_at_or_default(parent_x, parent_y) {
                                None => {
                                    // the source layer tile is unset
                                    // we need to call this method recursively
                                    // for the parent layer at the parent coordinates
                                    parent_layer.ensure_chunk_is_complete(parent_x, parent_y);
                                    // get the parent tile again. It should be set this time.
                                    parent_layer
                                        .get_at(parent_x, parent_y)
                                        .expect("Parent chunk tile is not set.")
                                }
                                Some(t) => t,
                            }
                        };
                        // set the tile vale in the chunk from the parent tile value
                        let _ = chunk.set_at(this_layer_x, this_layer_y, parent_tile);
                    }
                }
            }
        }
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
            self.tile_bounds.width as isize,
            self.tile_bounds.height as isize,
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
