use crate::bounds::Bounds;
use crate::chunk_generator::chunk_generator_trait::ChunkGenerator;
use crate::chunk_layer::{ChunkLayer, TIndex};
use crate::chunk_seed_utils::chunk_seed_utils_impl::create_seed_from_guid_bytes_x_y;
use indexmap::IndexMap;
use rand::prelude::StdRng;
use rand_core::{RngCore, SeedableRng};

pub(crate) struct ChunkSeededInterpolator;

impl ChunkGenerator for ChunkSeededInterpolator {
    fn generate_chunk_from_parent(
        &self,
        chunk_bounds: Bounds,
        child_layer: &mut ChunkLayer,
        parent_tiles: IndexMap<(isize, isize), TIndex>,
    ) {
        // get the layer relative tile coordinates for the area that needs
        // to be set in this chunk
        let (this_layer_x0, this_layer_y0, this_layer_x1, this_layer_y1) =
            chunk_bounds.get_bound_coords(true);

        let w = 2 + this_layer_x1 - this_layer_x0;
        let h = 2 + this_layer_y1 - this_layer_y0;
        let s = (w * h) as usize;
        let mut tiles = IndexMap::with_capacity(s);

        // first pass - just double the chunks
        for this_layer_y in this_layer_y0..this_layer_y1 {
            for this_layer_x in this_layer_x0..this_layer_x1 {
                // get the parent tile
                let parent_tile = parent_tiles[&(this_layer_x, this_layer_y)];
                // check existing tile
                let tile_value = child_layer
                    .get_at(this_layer_x, this_layer_y)
                    .unwrap_or(parent_tile);
                // set the tile vale in the chunk from the parent tile value
                // let ix = (this_layer_x + 1) + (this_layer_y + 1) * w;
                tiles.insert((this_layer_x, this_layer_y), tile_value);
            }
        }

        // let's get the random seed.
        let seed = {
            let mut result = [0u8; 32];
            let seed2 = create_seed_from_guid_bytes_x_y(
                &child_layer.layer_guid_bytes,
                this_layer_x0,
                this_layer_y0,
            );
            let seed1 = child_layer.manager_guid_bytes;
            result[..16].copy_from_slice(&seed1);
            result[16..].copy_from_slice(&seed2);
            result
        };
        let mut rng = StdRng::from_seed(seed);

        // second pass - copy some chunks from their neighbors
        for this_layer_y in (this_layer_y0 - 1..this_layer_y1 + 1).step_by(2) {
            for this_layer_x in this_layer_x0 - 1..this_layer_x1 + 1
            /*.step_by(2)*/
            {
                let dx = (rng.next_u32() % 3) as isize - 1;
                let dy = (rng.next_u32() % 3) as isize - 1;
                // get the offset tile
                // let offset_tile = child_layer.get_at(this_layer_x + dx, this_layer_y + dy);
                let offset_tile = tiles.get(&(this_layer_x + dx, this_layer_y + dy));
                match offset_tile {
                    None => {}
                    Some(tile_value) => {
                        // let _ = child_layer.set_at(this_layer_x, this_layer_y, tile_value);
                        // let ix = (this_layer_x + 1) + (this_layer_y + 1) * w;
                        // tiles[ix] = tile_value;
                        tiles.insert((this_layer_x, this_layer_y), *tile_value);
                    }
                }
            }
        }

        // third pass - copy some chunks from their neighbors
        for this_layer_y in this_layer_y0 - 1..this_layer_y1 + 1
        /*.step_by(2)*/
        {
            for this_layer_x in (this_layer_x0 - 1..this_layer_x1 + 1).step_by(2) {
                let dx = (rng.next_u32() % 3) as isize - 1;
                let dy = (rng.next_u32() % 3) as isize - 1;
                // get the offset tile
                // let offset_tile = child_layer.get_at(this_layer_x + dx, this_layer_y + dy);
                let offset_tile = tiles.get(&(this_layer_x + dx, this_layer_y + dy));
                match offset_tile {
                    None => {}
                    Some(tile_value) => {
                        // let _ = child_layer.set_at(this_layer_x, this_layer_y, tile_value);
                        // let ix = (this_layer_x + 1) + (this_layer_y + 1) * w;
                        // tiles[ix] = tile_value;
                        tiles.insert((this_layer_x, this_layer_y), *tile_value);
                    }
                }
            }
        }

        // TODO! Add tests to verify that map is identical no matter which
        // TODO! order it is filled. tl -> br, br -> tl
        assert!(tiles.len() <= s);

        // now fill the chunk from the tiles
        for ((tx, ty), tile_value) in tiles.drain(..) {
            let _ = child_layer.set_at(tx, ty, tile_value);
        }
    }
}
