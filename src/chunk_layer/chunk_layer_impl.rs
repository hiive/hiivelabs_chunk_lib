use crate::bounds::Bounds;
use crate::chunk::Chunk;
use miniz_oxide::deflate::compress_to_vec;
use schnellru::{ByLength, LruMap};
use smallvec::SmallVec;
use std::cell::RefCell;
use std::rc::Rc;

pub type TIndex = usize;

pub struct ChunkLayer {
    pub(crate) layer_id: usize,
    pub(crate) layer_chunk_lru_cache_size: u32,
    pub(crate) bounds: Bounds, // this is in chunk-coords, not tile-coords

    pub(crate) width_in_tiles: usize,
    pub(crate) height_in_tiles: usize,
    pub(crate) chunk_padding_in_tiles: usize,
    pub(crate) chunk_width: usize,
    pub(crate) chunk_height: usize,
    pub(crate) chunks: RefCell<LruMap<isize, Chunk>>,
    pub(crate) parent_layer: Rc<RefCell<Option<ChunkLayer>>>,
}

impl ChunkLayer {
    pub(crate) fn make_layer_rc(layer: Option<ChunkLayer>) -> Rc<RefCell<Option<ChunkLayer>>> {
        Rc::new(RefCell::new(layer))
    }

    pub fn new(
        parent_layer: Rc<RefCell<Option<ChunkLayer>>>,
        layer_id: usize,
        layer_chunk_lru_cache_size: u32,
        width_in_chunks: usize,
        height_in_chunks: usize,
        chunk_padding_in_tiles: usize,
        chunk_width: usize,
        chunk_height: usize,
    ) -> Self {
        let width_in_tiles = width_in_chunks * chunk_width;
        let height_in_tiles = height_in_chunks * chunk_height;
        let chunks = {
            let hashmap_capacity = width_in_chunks * height_in_chunks;
            // let hashmap = HashMap::with_capacity(hashmap_capacity);
            let mut hashmap = LruMap::new(ByLength::new(layer_chunk_lru_cache_size));
            hashmap.reserve_or_panic(hashmap_capacity);
            // hashmap.resize_with(vec_size, Default::default);
            RefCell::new(hashmap)
        };

        // let chunk_store = World::new();
        Self {
            layer_id,
            layer_chunk_lru_cache_size,
            bounds: Bounds {
                x: 0,
                y: 0,
                width: width_in_chunks,
                height: height_in_chunks,
                padding: 0,
                assert_on_out_of_bounds: false,
            },
            width_in_tiles,
            height_in_tiles,
            chunk_padding_in_tiles,
            chunk_width,
            chunk_height,
            chunks,
            parent_layer,
            // manager:None,
            // owned_values: Vec::with_capacity(width_in_tiles * height_in_tiles), // chunk_store
        }
    }

    pub(crate) fn get_hash_index_for_chunk_coords(&self, cx: isize, cy: isize) -> Option<isize> {
        if cx < 0 || cx >= self.bounds.width as isize || cy < 0 || cy >= self.bounds.height as isize
        {
            return None;
        }

        let ix = cx + (cy * self.bounds.width as isize);
        Some(ix)
    }

    fn get_parent_chunks_for_expansion(&self, parent_tx: isize, parent_ty: isize) -> Vec<Chunk> {
        // Directly borrow `parent_layer` for longer scope.
        let parent_layer_ref = self.parent_layer.borrow();
        let parent_layer = parent_layer_ref
            .as_ref()
            .expect("No parent layer for this layer");

        // Now `parent_layer` can be used safely within this scope.
        let indices = parent_layer.get_chunk_indices_for_tile_coords(parent_tx, parent_ty);
        let chunks = indices
            .iter()
            .map(|i| {
                let mut chunks = parent_layer.chunks.borrow_mut();
                let parent_chunk = chunks.get(&i.0).expect("No chunk found.");
                parent_chunk.clone()
            })
            .collect();
        chunks
    }

    pub(crate) fn populate_chunk_at(&mut self, dest_layer_tx: isize, dest_layer_ty: isize) -> bool {
        if self.parent_layer.borrow().is_none() {
            return false;
        }

        let parent_chunks =
            self.get_parent_chunks_for_expansion(dest_layer_tx / 2, dest_layer_ty / 2);
        self.populate_chunk_from_parent(&parent_chunks, dest_layer_tx, dest_layer_ty);
        true
    }

    pub(crate) fn populate_chunk_from_parent(
        &mut self,
        parent_chunks: &Vec<Chunk>,
        dest_layer_tx: isize,
        dest_layer_ty: isize,
    ) {
        // calculate the bounds of the chunk at this layer in terms of the layer above.
        /*
        let dest_chunk_indices =
            self.get_chunk_indices_for_tile_coords(dest_layer_tx, dest_layer_ty);
        for (dest_chunk_index, _) in dest_chunk_indices {
            let dest_chunk = &self.chunks.get(&dest_chunk_index);
            match dest_chunk {
                Some(_) => {}
                None => {}
            }
        }

         */

        // CHANGE THE FOLLOWING
        // get the extent of the previous layer chunk

        /*
        let prev_chunk_indices: Vec<Option<&Chunk>> = {
            let parent_layer_opt = self.parent_layer.borrow();
            let parent_layer = parent_layer_opt.as_ref().unwrap();
            let indices = parent_layer.get_chunk_indices_for_tile_coords(dest_layer_tx / 2, dest_layer_ty / 2);
            indices.iter().map(|i| parent_layer.get_chunk_by_index(i.0)).collect()
        };
         */

        //let parent_chunks = self.get_parent_chunks_for_expansion(dest_layer_tx / 2, dest_layer_ty / 2);

        for source_chunk in parent_chunks {
            // get the bounds of the source chunk
            let source_x = source_chunk.bounds.x;
            let source_y = source_chunk.bounds.y;
            let source_w = source_chunk.bounds.width as isize + source_x;
            let source_h = source_chunk.bounds.height as isize + source_y;
            for sy in source_y..source_h {
                for sx in source_x..source_w {
                    let o_v = source_chunk.get_at(sx, sy);
                    if let Some(v) = o_v {
                        let source_tile_value = source_chunk.tiles[v]
                            //.as_ref()
                            .expect("Source Chunk Has no value set");
                        //.value;
                        // populate the dest chunk
                        for dy in 0_isize..2 {
                            for dx in 0_isize..2 {
                                self.set_at(
                                    dest_layer_tx + dx,
                                    dest_layer_ty + dy,
                                    source_tile_value,
                                )
                            }
                        }
                    }
                }
            }
        }

        // let chunk_indices = self.get_chunk_indices_for_tile_coords(dest_layer_tx, dest_layer_ty);
    }

    pub(crate) fn is_chunk_border_coord(&self, tx: isize, ty: isize) -> (bool, bool) {
        (
            // has to be inside outer bounds, and also withing padding between chunks
            tx > 0 && tx <= self.width_in_tiles as isize && tx % self.chunk_width as isize == 0,
            ty > 0 && ty <= self.height_in_tiles as isize && ty % self.chunk_height as isize == 0,
        )
    }

    pub(crate) fn tile_coords_to_chunk_coords(&self, tx: isize, ty: isize) -> (isize, isize) {
        (
            tx / self.chunk_width as isize,
            ty / self.chunk_height as isize,
        )
    }

    pub(crate) fn chunk_coords_to_tile_coords(&self, cx: isize, cy: isize) -> (isize, isize) {
        (
            cx * self.chunk_width as isize,
            cy * self.chunk_height as isize,
        )
    }

    pub(crate) fn get_chunk_indices_for_tile_coords(
        &self,
        tx: isize,
        ty: isize,
    ) -> SmallVec<[(isize, (isize, isize)); 4]> {
        // there are two main possibilities here.
        // 1. It's a chunk boundary, so multiple chunks will be returned.
        // 2. It's within a chunk, so only one chunk will be returned
        let mut chunk_indices = SmallVec::with_capacity(4);
        let padding = self.chunk_padding_in_tiles as isize;
        let width = self.width_in_tiles as isize;
        let height = self.width_in_tiles as isize;

        // short circuit if well out of bounds
        if tx < -padding || tx >= width + padding || ty < -padding || ty >= height + padding {
            // completely out of bounds - no matching chunks
            return chunk_indices;
        }

        // figure out a chunk that these coords are in
        let (cx, cy) = self.tile_coords_to_chunk_coords(tx, ty);
        // #[cfg(debug_assertions)]
        // println!("ChunkLayer::get_chunk_indices_for_tile_coords: tile_coords_to_chunk_coords: ({tx}, {ty}) -> ({cx}, {cy})");

        // get main chunk index
        // let (chunk_ix, _, _) = self.bounds.get_index_for_coords(cx, cy);
        let chunk_ix = self.get_hash_index_for_chunk_coords(cx, cy);
        // add the chunk index if it's valid
        match chunk_ix {
            Some(ix) => {
                // chunk index valid
                chunk_indices.push((ix, (cx, cy)));
            }
            None => {
                // if the chunk index is not valid, then we need to
                // take this into account... todo!
            }
        }

        // /*
        // are they on a boundary?
        let (x_is_boundary, y_is_boundary) = self.is_chunk_border_coord(tx, ty);

        if !x_is_boundary && !y_is_boundary {
            // easiest case; these coords are in one chunk only.
            // we can do a short circuit return
            return chunk_indices;
        } else {
            let (c_tx, c_ty) = self.chunk_coords_to_tile_coords(cx, cy);
            let chunk_r = c_tx + self.chunk_width as isize;
            let chunk_b = c_ty + self.chunk_width as isize;
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
            // check right
            if x_is_boundary {
                // it's on the x boundary, so do we need the chunk to the right?
                self.add_boundary_chunk_if_in_bounds(&mut chunk_indices, cx + d_cx, cy);
            }
            // check up/down
            if y_is_boundary {
                // it's on the y boundary, so do we need the chunk to the bottom?
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
        chunk_indices: &mut SmallVec<[(isize, (isize, isize)); 4]>,
        cx: isize,
        cy: isize,
    ) {
        // let (chunk_ix, _, _) = self.bounds.get_index_for_coords(cx, cy);
        let chunk_ix = self.get_hash_index_for_chunk_coords(cx, cy);
        // check we are in bounds
        // if self.bounds.is_in_bounds(chunk_ix) {
        if let Some(ix) = chunk_ix {
            if self.bounds.is_in_bounds(ix) {
                chunk_indices.push((ix, (cx, cy)))
            }
        }
    }

    // pub(crate) fn set_many_at(&mut self, values: &mut Vec<(isize, isize, TIndex)>) {
    //     let mut extracted_values: Vec<(SmallVec<[usize; 4]>, isize, isize, TIndex)> = values
    //         .drain(..)
    //         .map(|xyv| {
    //             let (tx, ty, v) = xyv;
    //             let ix = self.get_chunk_indices_for_tile_coords(tx, ty);
    //             (ix, tx, ty, v)
    //         })
    //         .collect();
    //
    //     // cache the owned values
    //     // let mut owned_values = Vec::<usize>::with_capacity(extracted_values.len());
    //
    //     for (chunk_ixs, tx, ty, value) in extracted_values.drain(..) {
    //         // get index of added value
    //         // let v_ix = self.owned_values.len() + owned_values.len();
    //         // save value
    //         // owned_values.push(value);
    //
    //         // set values
    //         for chunk_ix in chunk_ixs {
    //             match self.chunks[chunk_ix] {
    //                 Some(ref mut chunk) => {
    //                     chunk.set_at(tx, ty, value);
    //                 }
    //                 None => {
    //                     // need to create this chunk
    //                     let (cx, cy) = self.tile_coords_to_chunk_coords(tx, ty);
    //                     let (c_tx, c_ty) = self.chunk_coords_to_tile_coords(cx, cy);
    //                     let mut new_chunk = Chunk::new(
    //                         c_tx,
    //                         c_ty,
    //                         self.chunk_width,
    //                         self.chunk_height,
    //                         self.chunk_padding_in_tiles,
    //                     );
    //                     new_chunk.set_at(tx, ty, value);
    //                     self.chunks[chunk_ix] = Some(new_chunk);
    //                 }
    //             }
    //         }
    //     }
    //
    //     // append the cached own values to the main cache
    //     // let ov = &mut self.owned_values;
    //     // owned_values.append(ov);
    // }

    pub(crate) fn ensure_chunk_exists(&mut self, tx: isize, ty: isize) {
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);

        let mut chunks = self.chunks.borrow_mut();
        for (chunk_ix, (cx, cy)) in chunk_ixs {
            match chunks.get(&chunk_ix) {
                Some(_) => {
                    // already exists. No action required.
                }
                None => {
                    let (c_tx, c_ty) = self.chunk_coords_to_tile_coords(cx, cy);

                    // #[cfg(debug_assertions)]
                    // println!("ChunkLayer::ensure_chunk_exists: chunk_coords_to_tile_coords: ({cx}, {cy}) -> ({c_tx}, {c_ty})");

                    let new_chunk = Chunk::new(
                        c_tx,
                        c_ty,
                        self.chunk_width,
                        self.chunk_height,
                        self.chunk_padding_in_tiles,
                    );
                    if chunks.len() == self.layer_chunk_lru_cache_size as usize {
                        // the cache is full.
                        // pop the oldest
                        if let Some((ix, oldest_chunk)) = chunks.peek_oldest() {
                            // need to store this chunk
                            self.store_chunk(ix, oldest_chunk);
                        }
                    }
                    chunks.insert(chunk_ix, new_chunk);
                }
            }
        }
    }
    fn store_chunk(&self, chunk_ix: &isize, chunk: &Chunk) {
        // todo - make sure we store sizes in the db
        let encoded = bitcode::encode(chunk);
        let compressed = compress_to_vec(encoded.as_slice(), 6);
    }

    pub fn set_at(&mut self, tx: isize, ty: isize, value: TIndex) {
        // /*
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);

        // #[cfg(debug_assertions)]
        // println!("ChunkLayer::set_at: get_chunk_indices_for_tile_coords: ({tx}, {ty}) -> {chunk_ixs:?}");

        // save value
        self.ensure_chunk_exists(tx, ty);

        // let value_ref = self.owned_values.last().unwrap();
        for (chunk_ix, _) in chunk_ixs {
            let mut chunks = self.chunks.borrow_mut();
            let chunk = chunks.get(&chunk_ix).unwrap(); // we know the chunk exists.

            // #[cfg(debug_assertions)]
            // println!("ChunkLayer::set_at: chunk[ix]: ({chunk_ix})");

            chunk.set_at(tx, ty, value);
        }
    }

    pub fn get_at(&self, tx: isize, ty: isize) -> Option<TIndex> {
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        if chunk_ixs.is_empty() {
            return None;
        }
        let (chunk_ix, _) = chunk_ixs[0];
        let mut chunks = self.chunks.borrow_mut();
        let chunk = chunks.get(&chunk_ix);

        match chunk {
            Some(chunk) => chunk.get_at(tx, ty),
            _ => None, // Either the index is out of bounds or the Option<ChunkTile<T>> is None
        }
    }

    /*
    pub fn get_chunk_at(&'a self, x: isize, y: isize) -> Option<&'a Chunk<T>> {
        let chunk_coords = self.get_chunk_indices_for_tile_coords(x, y);
        for chunk_ix in chunk_coords {
            // let (ix, _, _) = self.bounds.get_index_for_coords(x, y);
            let chunk = &self.chunks[chunk_ix];
            if chunk.is_none() {
                // needs to adjust coords to chunk boundary.
                //self.chunks[ix] = Chunk::new()
            }
        }
        None
        // self.chunks[0].as_ref() // <-- this is how to get value in correct form
    }
     */
}
