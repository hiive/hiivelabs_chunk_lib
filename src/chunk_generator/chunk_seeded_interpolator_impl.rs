use crate::bounds::Bounds;
use crate::chunk_generator::chunk_generator_trait::ChunkGenerator;
use crate::chunk_layer::{ChunkLayer, TIndex};
use crate::chunk_seed_utils::chunk_seed_utils_impl::create_seed_from_guid_bytes_x_y;
use indexmap::IndexMap;
use rand::prelude::StdRng;
use rand_core::SeedableRng;

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
        let mut _rng = StdRng::from_seed(seed);

        // set up a vector to store child tiles.
        let mut child_tiles = IndexMap::with_capacity(chunk_bounds.get_tile_count(true));

        // loop over the layer tile coordinates

        // 1. first pass - copy in source tiles from layer above.
        // We need to get values outside the chunk bounds
        for this_layer_y in this_layer_y0 - 1..=this_layer_y1 {
            for this_layer_x in this_layer_x0 - 1..=this_layer_x1 {
                // get the parent tile
                let parent_tile_opt = parent_tiles.get(&(this_layer_x, this_layer_y));
                if parent_tile_opt.is_none() {
                    continue;
                }
                let parent_tile = *parent_tile_opt.unwrap();
                // check existing tile
                let (tile_value, is_fixed) = {
                    match child_layer.get_at(this_layer_x, this_layer_y) {
                        Some(existing_tile) => {
                            if existing_tile != parent_tile {
                                // set the tile vale in the chunk from the parent tile value
                                // test
                                let _ = child_layer.set_at(this_layer_x, this_layer_y, parent_tile);
                                // end test
                                (existing_tile, true)
                            } else {
                                (parent_tile, false)
                            }
                        }
                        None => (parent_tile, false),
                    }
                };
                child_tiles.insert((this_layer_x, this_layer_y), (tile_value, is_fixed));
            }
        }
        // we now have a vector containing all the child tiles, including which ones are fixed.
        // of the format (tx, ty) -> (t_index, is_fixed).
        // let dx = 1_isize;
        // let dy = chunk_bounds.get_width(true) + 2;  // +2 because we sampled
        //                                                           // outside of chunk bounds

        // 2. second pass - diagonal sample.
        for this_layer_y in (this_layer_y0..this_layer_y1).step_by(2) {
            for this_layer_x in (this_layer_x0..this_layer_x1).step_by(2) {
                let _ = child_layer.set_at(this_layer_x, this_layer_y, 0);
            }
        }

        // 3. third pass - orthogonal sample
    }
}
