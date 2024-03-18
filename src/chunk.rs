use crate::bounds::Bounds;
use crate::chunk_tile::ChunkTile;
use std::fmt::Debug;

pub struct Chunk<'c, T> {
    pub(crate) bounds: Bounds,
    chunk_width: usize,
    chunk_height: usize,
    pub(crate) tiles: Vec<Option<ChunkTile<'c, T>>>,
}

impl<'c, T: Debug> Chunk<'c, T> {
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

    pub fn get_at(&self, tx: isize, ty: isize) -> Option<&'c T> {
        let (ix, _, _) = self.bounds.get_index_for_coords(tx, ty);
        let tile = self.tiles[ix as usize].as_ref();

        match tile {
            Some(chunk_tile) => Some(&chunk_tile.value), // Return a reference to the value
            _ => None, // Either the index is out of bounds or the Option<ChunkTile<T>> is None
        }
    }

    pub fn set_at(&mut self, tx: isize, ty: isize, value: &'c T) {
        let (ix, tx, ty) = self.bounds.get_index_for_coords(tx, ty);
        let ix = ix as usize;
        match self.tiles[ix] {
            Some(ref mut tile) => {
                // If there is already a ChunkTile, update its value
                tile.value = value;
            }
            None => {
                // If there is no ChunkTile, insert a new one
                self.tiles[ix] = Some(ChunkTile::new(tx, ty, value));
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
// tests
////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
mod chunk_tests {
    #[cfg(test)]
    use crate::chunk::Chunk;

    #[test]
    fn chunk_creation() {
        let chunk = Chunk::<i32>::new(0, 0, 10, 10, 1);
        assert_eq!(chunk.bounds.width, 10);
        assert_eq!(chunk.bounds.height, 10);
        assert_eq!(chunk.bounds.padding, 1);
        // Ensure the vector is correctly sized with padding
        assert_eq!(chunk.tiles.len(), 144); // (10 + 2*1) * (10 + 2*1)
    }

    #[test]
    fn get_at_for_empty_tile() {
        let chunk = Chunk::<i32>::new(0, 0, 10, 10, 1);
        assert!(chunk.get_at(5, 5).is_none());
    }

    #[test]
    fn set_at_for_occupied_tile() {
        let mut chunk = Chunk::new(0, 0, 10, 10, 1);
        chunk.set_at(5, 5, &42); // Assuming ChunkTile takes an i32 for this example
        assert_eq!(*chunk.get_at(5, 5).unwrap(), 42);
        chunk.set_at(5, 5, &43); // Assuming ChunkTile takes an i32 for this example
        assert_eq!(*chunk.get_at(5, 5).unwrap(), 43);
    }

    #[test]
    fn set_and_get_tile() {
        let mut chunk = Chunk::new(0, 0, 10, 10, 1);
        chunk.set_at(5, 5, &42); // Assuming ChunkTile takes an i32 for this example
        assert_eq!(*chunk.get_at(5, 5).unwrap(), 42);
    }

    #[test]
    #[should_panic(expected = "Chunk coordinates are out of bounds")]
    fn set_at_out_of_bounds() {
        let mut chunk = Chunk::new(0, 0, 10, 10, 1);
        chunk.set_at(50, 50, &42); // This should panic
    }

    #[test]
    fn it_works() {
        let mut chunk: Chunk<u32> = Chunk::new(10, 10, 20, 10, 1);

        // chunk.init_chunk_tile(9, 9, 99);
        chunk.set_at(9, 9, &101);

        {
            let test = chunk.get_at(9, 9);
            // let tv = test.unwrap_or(&u32::MAX);

            println!("{test:?}");
        }

        chunk.set_at(9, 9, &102);
        let test = chunk.get_at(9, 9);

        println!("{test:?}");
        //let tv = test.unwrap_or(&u32::MAX);
    }
}
