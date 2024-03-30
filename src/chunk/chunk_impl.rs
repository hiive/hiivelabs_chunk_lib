use crate::bounds::Bounds;
use crate::chunk_layer::TIndex;
use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct Chunk {
    pub(crate) bounds: Bounds,
    pub(crate) chunk_width: usize,
    pub(crate) chunk_height: usize,
    pub(crate) tiles: Vec<Option<TIndex>>,
}

impl Chunk {
    pub fn new(x: isize, y: isize, width: usize, height: usize, padding: usize) -> Self {
        let chunk_width = width + 2 * padding;
        let chunk_height = height + 2 * padding;
        let tiles = {
            let vec_size = chunk_width * chunk_height;
            let mut vec = Vec::with_capacity(vec_size);
            vec.resize_with(vec_size, Default::default);
            vec
        };

        Self {
            bounds: Bounds {
                x,
                y,
                width,
                height,
                padding,
                assert_on_out_of_bounds: true,
            },
            chunk_width,
            chunk_height,
            tiles,
        }
    }

    pub fn get_at(&self, tx: isize, ty: isize) -> Option<TIndex> {
        let (ix, _, _) = self.bounds.get_index_for_coords(tx, ty);
        let tile = self.tiles[ix as usize].as_ref();

        match tile {
            Some(chunk_tile) => Some(*chunk_tile), // Return a reference to the value
            _ => None, // Either the index is out of bounds or the Option<ChunkTile<T>> is None
        }
    }

    pub fn is_complete(&self) -> bool {
        let any_nones = self.tiles.iter().any(Option::is_none);
        !any_nones
    }

    pub fn set_at(&mut self, tx: isize, ty: isize, value: TIndex) {
        let (ix, _, _) = self.bounds.get_index_for_coords(tx, ty);
        // #[cfg(debug_assertions)]
        // println!("Chunk::set_at: get_index_for_coords: ({tx}, {ty}) -> ({ix})");
        let ix = ix as usize;
        match self.tiles[ix] {
            Some(ref mut tile) => {
                // If there is already a tile, update its value
                *tile = value;
            }
            None => {
                // If there is no tile, insert a new one
                self.tiles[ix] = Some(value);
            }
        }
    }
}