use crate::bounds::Bounds;
use crate::chunk::Chunk;
// use hecs::World;
use crate::ChunkManager;
use smallvec::SmallVec;
use std::option::Option;
use std::sync::Arc;
use crate::chunk_tile::ChunkTile;

pub struct ChunkLayer<T> {
    layer_id: usize,
    bounds: Bounds, // this is in chunks, not tile-coords

    width_in_tiles: usize,
    height_in_tiles: usize,
    chunk_padding_in_tiles: usize,
    chunk_width: usize,
    chunk_height: usize,
    chunks: Vec<Option<Chunk>>,
    // manager: Option<&'m Arc<&'m ChunkManager<'m, T>>>,
    owned_values: Vec<T>,
}


impl<T: std::fmt::Debug> ChunkLayer<T> {
    pub fn new(
        // manager: &'m ChunkManager<'m, T>,
        layer_id: usize,
        width_in_chunks: usize,
        height_in_chunks: usize,
        chunk_padding_in_tiles: usize,
        chunk_width: usize,
        chunk_height: usize,
    ) -> Self {
        let width_in_tiles = width_in_chunks * chunk_width;
        let height_in_tiles = height_in_chunks * chunk_height;
        let chunks = {
            let vec_size = width_in_chunks * height_in_chunks;
            let mut vec = Vec::with_capacity(vec_size);
            vec.resize_with(vec_size, Default::default);
            vec
        };

        // let chunk_store = World::new();
        Self {
            layer_id,
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
            // manager:None,
            owned_values: Vec::with_capacity(width_in_tiles * height_in_tiles), // chunk_store
        }
    }

    pub(crate) fn populate_chunk_at(&mut self,
                                    prev_layer: &ChunkLayer<T>,
                                    current_layer_tx: isize, current_layer_ty: isize) {

        // get the extent of the previous layer chunk
        let prev_chunk_indices = prev_layer
            .get_chunk_indices_for_tile_coords(current_layer_tx / 2, current_layer_ty /2);

        for c_ix in prev_chunk_indices {
            let prev_chunk = prev_layer.chunks[c_ix].as_ref().unwrap();

            let source_x = prev_chunk.bounds.x;
            let source_y = prev_chunk.bounds.y;
            let source_w = prev_chunk.bounds.width as isize + source_x;
            let source_h = prev_chunk.bounds.height as isize + source_y;
            let owned_tiles = &mut self.owned_values;
            for sy in source_y..source_h {
                for sx in source_x..source_w {
                    let o_v = prev_chunk.get_at(sx, sy);
                    match(o_v)
                    {
                        None => {}
                        Some(v) => {
                            let source_tile = prev_chunk.tiles[v].as_ref().unwrap();
                            let source_tile_value = &prev_layer.owned_values[source_tile.value];
                            for dy in 0_isize..2 {
                                for dx in 0_isize..2 {

                                }
                            }

                        }
                    }
                }
            }
        }


        let chunk_indices = self.get_chunk_indices_for_tile_coords(
            current_layer_tx,
            current_layer_ty
        );

    }

    /*
    pub(crate) fn set_manager(&mut self, manager: &'m Arc<&ChunkManager<T>>) {
        self.manager = Option::from(manager);
    }
     */

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
    ) -> SmallVec<[usize; 4]> {
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
        let (mut cx, mut cy) = self.tile_coords_to_chunk_coords(tx, ty);

        // are they on a boundary?
        let (mut x_is_boundary, mut y_is_boundary) = self.is_chunk_border_coord(tx, ty);

        // if we are on a boundary, let's default to the chunk to the left/top
        if x_is_boundary && cx > 0 {
            cx -= 1;
            // if it's the right hand edge, unset the flag
            x_is_boundary = tx < width;
        }
        if y_is_boundary && cy > 0 {
            cy -= 1;
            // if it's the bottom edge, unset the flag
            y_is_boundary = ty < height;
        }

        // get main chunk index
        let (chunk_ix, _, _) = self.bounds.get_index_for_coords(cx, cy);
        // check we are in bounds
        if self.bounds.is_in_bounds(chunk_ix) {
            chunk_indices.push(chunk_ix as usize)
        };
        if !x_is_boundary && !y_is_boundary {
            // easiest case; these coords are in one chunk only.
            // we can do a short circuit return
            return chunk_indices;
        } else {
            // check right
            if x_is_boundary {
                // it's on the x boundary, so do we need the chunk to the right?
                self.add_boundary_chunk_if_in_bounds(&mut chunk_indices, cx + 1, cy);
            }
            // check up/down
            if y_is_boundary {
                // it's on the y boundary, so do we need the chunk to the bottom?
                self.add_boundary_chunk_if_in_bounds(&mut chunk_indices, cx, cy + 1);
            }
            // check corners
            if x_is_boundary && y_is_boundary {
                self.add_boundary_chunk_if_in_bounds(&mut chunk_indices, cx + 1, cy + 1);
            }
        }

        // let (ix, cx, cy) = self.bounds.get_index_for_coords();
        // check for boundary coords

        chunk_indices
    }

    fn add_boundary_chunk_if_in_bounds(
        &self,
        chunk_indices: &mut SmallVec<[usize; 4]>,
        cx: isize,
        cy: isize,
    ) {
        let (chunk_ix, _, _) = self.bounds.get_index_for_coords(cx, cy);
        // check we are in bounds
        if self.bounds.is_in_bounds(chunk_ix) {
            chunk_indices.push(chunk_ix as usize)
        };
    }

    pub(crate) fn set_many_at(&mut self, values: &mut Vec<(isize, isize, T)>) {
        let mut extracted_values: Vec<(SmallVec<[usize; 4]>, isize, isize, T)> = values
            .drain(..)
            .map(|xyv| {
                let (tx, ty, v) = xyv;
                let ix = self.get_chunk_indices_for_tile_coords(tx, ty);
                (ix, tx, ty, v)
            })
            .collect();

        // cache the owned values
        let mut owned_values = Vec::<T>::with_capacity(extracted_values.len());

        for (chunk_ixs, tx, ty, value) in extracted_values.drain(..) {
            // get index of added value
            let v_ix = self.owned_values.len() + owned_values.len();
            // save value
            owned_values.push(value);

            // set values
            for chunk_ix in chunk_ixs {
                match self.chunks[chunk_ix] {
                    Some(ref mut chunk) => {
                        chunk.set_at(tx, ty, v_ix);
                    }
                    None => {
                        // need to create this chunk
                        let (cx, cy) = self.tile_coords_to_chunk_coords(tx, ty);
                        let (c_tx, c_ty) = self.chunk_coords_to_tile_coords(cx, cy);
                        let mut new_chunk = Chunk::new(
                            c_tx,
                            c_ty,
                            self.chunk_width,
                            self.chunk_height,
                            self.chunk_padding_in_tiles,
                        );
                        new_chunk.set_at(tx, ty, v_ix);
                        self.chunks[chunk_ix] = Some(new_chunk);
                    }
                }
            }
        }

        // append the cached own values to the main cache
        let ov = &mut self.owned_values;
        owned_values.append(ov);
    }

    pub fn set_at(&mut self, tx: isize, ty: isize, value: T) {
        // /*
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);

        // save value
        self.owned_values.push(value);

        // let value_ref = self.owned_values.last().unwrap();
        for chunk_ix in chunk_ixs {
            match self.chunks[chunk_ix] {
                Some(ref mut chunk) => {
                    chunk.set_at(tx, ty, self.owned_values.len());
                }
                None => {
                    // need to create this chunk
                    let (cx, cy) = self.tile_coords_to_chunk_coords(tx, ty);
                    let (c_tx, c_ty) = self.chunk_coords_to_tile_coords(cx, cy);
                    let mut new_chunk = Chunk::new(
                        c_tx,
                        c_ty,
                        self.chunk_width,
                        self.chunk_height,
                        self.chunk_padding_in_tiles,
                    );
                    new_chunk.set_at(tx, ty, self.owned_values.len());
                    self.chunks[chunk_ix] = Some(new_chunk);
                }
            }
        }
    }

    pub fn get_at(&self, tx: isize, ty: isize) -> Option<&T> {
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        if chunk_ixs.len() == 0 {
            return None;
        }
        let chunk_ix = chunk_ixs[0];

        let chunk = self.chunks[chunk_ix].as_ref();

        match chunk {
            Some(chunk) => {
                let chunk_ix = chunk.get_at(tx, ty);
                match chunk_ix {
                    Some(ix) => self.owned_values.get(ix),
                    None => None,
                }
            }
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

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
// tests
////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
mod chunk_layer_tests;
