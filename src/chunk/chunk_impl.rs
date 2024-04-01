use crate::bounds::Bounds;
use crate::chunk_layer::TIndex;
use bitcode::{Decode, Encode};
use uuid::Uuid;

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct Chunk {
    pub(crate) bounds: Bounds,
    pub(crate) chunk_width: usize,
    pub(crate) chunk_height: usize,
    pub(crate) tiles: Vec<Option<TIndex>>,
    pub(crate) guid_bytes: [u8; 16],
}

impl Chunk {
    pub(crate) fn new(
        x: isize,
        y: isize,
        width: usize,
        height: usize,
        padding: usize,
        chunk_guid: Uuid,
    ) -> Self {
        let chunk_width = width + 2 * padding;
        let chunk_height = height + 2 * padding;
        let tiles = {
            let vec_size = chunk_width * chunk_height;
            let mut vec = Vec::with_capacity(vec_size);
            vec.resize_with(vec_size, Default::default);
            vec
        };

        let guid_bytes = chunk_guid.as_bytes().to_owned();

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
            guid_bytes,
        }
    }

    pub(crate) fn get_at(&self, tx: isize, ty: isize) -> Option<TIndex> {
        let result = self.bounds.get_index_for_coords(tx, ty, true);
        match result {
            Ok((ix, _, _)) => self.get_by_index(ix),
            Err(msg) => {
                panic!("{msg}")
            }
        }
    }

    pub(crate) fn get_at_or_default(
        &self,
        tx: isize,
        ty: isize,
        default_value: Option<TIndex>,
    ) -> Option<TIndex> {
        if let Ok((ix, _, _)) = self.bounds.get_index_for_coords(tx, ty, false) {
            if self.bounds.is_index_in_bounds(ix) {
                // !("Chunk::get_at_or_default: ({tx}, {ty}) Got default value: {default_value:?}");
                return self.get_by_index(ix);
            }
        }
        default_value
    }

    fn get_by_index(&self, ix: isize) -> Option<TIndex> {
        if ix >= self.tiles.len() as isize {
            None
        } else {
            self.tiles[ix as usize]
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

    pub(crate) fn set_at(&mut self, tx: isize, ty: isize, value: TIndex) -> Result<(), &str> {
        // #[cfg(debug_assertions)]
        // println!("Chunk::set_at: get_index_for_coords: ({tx}, {ty}) -> ({ix})");

        let result = self.bounds.get_index_for_coords(tx, ty, true);
        match result {
            Ok((ix, _, _)) => {
                self.tiles[ix as usize] = Some(value);
                Ok(())
            }
            Err(msg) => Err(msg),
        }
    }
}
