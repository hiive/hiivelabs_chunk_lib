use crate::bounds::Bounds;
use crate::chunk::Chunk;
use hecs::World;
use smallvec::SmallVec;
use crate::TileMapDataSource;

pub struct ChunkLayer<'l, T> {
    layer_id: usize,
    bounds: Bounds, // this is in chunks, not tile-coords

    width_in_tiles: usize,
    height_in_tiles: usize,
    chunk_padding_in_tiles: usize,
    chunk_width: usize,
    chunk_height: usize,
    chunks: Vec<Option<Chunk<'l, T>>>,
    parent_layer: Option<&'l ChunkLayer<'l, T>>,
}

impl<'l, T: std::fmt::Debug> ChunkLayer<'l, T> {
    pub fn new(
        layer_id: usize,
        width_in_chunks: usize,
        height_in_chunks: usize,
        chunk_padding_in_tiles: usize,
        chunk_width: usize,
        chunk_height: usize,
        parent_layer: Option<&'l ChunkLayer<'l, T>>
    ) -> Self {
        let width_in_tiles = width_in_chunks * chunk_width;
        let height_in_tiles = height_in_chunks * chunk_height;
        let chunks = {
            let vec_size = width_in_chunks * height_in_chunks;
            let mut vec = Vec::with_capacity(vec_size);
            vec.resize_with(vec_size, Default::default);
            vec
        };

        let chunk_store = World::new();
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
            parent_layer
            // chunk_store
        }
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

    pub fn set_at(&mut self, tx: isize, ty: isize, value: &'l T) {
        // /*
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);

        for chunk_ix in chunk_ixs {
            match self.chunks[chunk_ix] {
                Some(ref mut chunk) => {
                    chunk.set_at(tx, ty, value);
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
                    new_chunk.set_at(tx, ty, value);
                    self.chunks[chunk_ix] = Some(new_chunk);
                }
            }
        }
    }

    pub fn get_at(&self, tx: isize, ty: isize) -> Option<&'l T> {
        let chunk_ixs = self.get_chunk_indices_for_tile_coords(tx, ty);
        if chunk_ixs.len() == 0 {
            return None;
        }
        let chunk_ix = chunk_ixs[0];

        let chunk = self.chunks[chunk_ix].as_ref();

        match chunk {
            Some(chunk) => chunk.get_at(tx, ty), // Return a reference to the value
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
mod chunk_layer_tests {
    #[cfg(test)]
    use super::ChunkLayer;

    #[test]
    fn test_is_chunk_border_coord() {
        let layer = ChunkLayer::<i32>::new(1, 10, 10, 1, 10, 10, None);
        assert_eq!(layer.is_chunk_border_coord(10, 10), (true, true));
        assert_eq!(layer.is_chunk_border_coord(5, 5), (false, false));
    }

    #[test]
    fn test_tile_coords_to_chunk_coords() {
        let layer = ChunkLayer::<i32>::new(1, 10, 10, 1, 10, 10, None);
        assert_eq!(layer.tile_coords_to_chunk_coords(15, 25), (1, 2));
    }

    #[test]
    fn test_chunk_coords_to_tile_coords() {
        let layer = ChunkLayer::<i32>::new(1, 10, 10, 1, 10, 10, None);
        assert_eq!(layer.chunk_coords_to_tile_coords(1, 2), (10, 20));
    }

    #[test]
    fn test_get_chunk_indices_for_tile_coords() {
        let layer = ChunkLayer::<i32>::new(1, 2, 2, 1, 10, 10, None);
        // Assuming Bounds::get_index_for_coords and Bounds::is_in_bounds are correctly implemented
        // and chunks are properly initialized in the layer.
        // This example assumes chunks are laid out linearly and checks for boundary conditions.
        // Adjust the logic based on how your chunks are indexed and stored.

        // at the top-left boundary
        let indices = layer.get_chunk_indices_for_tile_coords(0, 0);
        assert_eq!(indices.len(), 1);
        assert!(indices.contains(&0));

        // at the bottom right boundary
        let indices = layer.get_chunk_indices_for_tile_coords(20, 20);
        assert_eq!(indices.len(), 1);
        assert!(indices.contains(&3));

        // at the top right boundary
        let indices = layer.get_chunk_indices_for_tile_coords(20, 0);
        assert_eq!(indices.len(), 1);
        println!("{indices:?}");
        assert!(indices.contains(&1));

        // at the bottom left boundary
        let indices = layer.get_chunk_indices_for_tile_coords(0, 20);
        assert_eq!(indices.len(), 1);
        println!("{indices:?}");
        assert!(indices.contains(&2));

        // out of bounds (-ve)
        let indices = layer.get_chunk_indices_for_tile_coords(-5, -5);
        assert_eq!(indices.len(), 0);

        // out of bounds (+ve)
        let indices = layer.get_chunk_indices_for_tile_coords(25, 25);
        assert_eq!(indices.len(), 0);

        // Directly within a chunk
        let indices = layer.get_chunk_indices_for_tile_coords(11, 11);
        assert_eq!(indices.len(), 1);
        println!("{indices:?}");
        assert!(indices.contains(&3));

        // On a chunk boundary
        let indices = layer.get_chunk_indices_for_tile_coords(10, 10);
        // assert_eq!(main_chunk, main_chunk_2);
        assert_eq!(indices.len(), 4); // Expect multiple indices due to boundary condition
        println!("{indices:?}");
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
        assert!(indices.contains(&2));
        assert!(indices.contains(&3));

        // on a left edge
        let indices = layer.get_chunk_indices_for_tile_coords(10, 0);
        assert_eq!(indices.len(), 2);
        println!("{indices:?}");
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
    }

    // Add more tests as needed for other methods and edge cases.
}
