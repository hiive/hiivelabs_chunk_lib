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
    pub(crate) fn new(x: isize, y: isize, width: usize, height: usize, padding: usize) -> Self {
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
            },
            chunk_width,
            chunk_height,
            tiles,
        }
    }

    pub(crate) fn get_at(&self, tx: isize, ty: isize) -> Option<TIndex> {
        let (ix, _, _) = self.bounds.get_index_for_coords(tx, ty, true);
        self.get_by_index(ix)
    }

    pub(crate) fn get_at_or_default(
        &self,
        tx: isize,
        ty: isize,
        default_value: Option<TIndex>,
    ) -> Option<TIndex> {
        let (ix, _, _) = self.bounds.get_index_for_coords(tx, ty, false);
        if !self.bounds.is_in_bounds(ix) {
            // !("Chunk::get_at_or_default: ({tx}, {ty}) Got default value: {default_value:?}");
            return default_value;
        }
        self.get_by_index(ix)
    }

    fn get_by_index(&self, ix: isize) -> Option<TIndex> {
        let tile = self.tiles[ix as usize].as_ref();

        match tile {
            Some(chunk_tile) => Some(*chunk_tile), // Return a reference to the value
            _ => None, // Either the index is out of bounds or the Option<ChunkTile<T>> is None
        }
    }

    pub(crate) fn is_complete(&self) -> bool {
        // let any_nones = self.tiles.iter().any(Option::is_none);
        let any_nones = self.tiles.iter().any(|t| t.is_none());
        !any_nones
    }

    pub(crate) fn get_unset_tile_count(&self) -> usize {
        let none_count = self.tiles.iter().filter(|t| t.is_none()).count();
        none_count
    }

    pub(crate) fn set_at(&mut self, tx: isize, ty: isize, value: TIndex) {
        let (ix, _, _) = self.bounds.get_index_for_coords(tx, ty, true);
        // #[cfg(debug_assertions)]
        // println!("Chunk::set_at: get_index_for_coords: ({tx}, {ty}) -> ({ix})");
        // let ix = ix as usize;
        self.tiles[ix as usize] = Some(value);
    }
}
