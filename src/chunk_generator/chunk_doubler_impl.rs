use crate::bounds::Bounds;
use crate::chunk_generator::chunk_generator_trait::ChunkGenerator;
use crate::chunk_layer::{ChunkLayer, TIndex};
use indexmap::IndexMap;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

pub(crate) struct ChunkDoubler;

impl ChunkGenerator for ChunkDoubler {
    fn generate_chunk_from_parent(
        &self,
        chunk_bounds: Bounds,
        child_layer: &mut ChunkLayer,
        parent_tiles: IndexMap<(isize, isize), TIndex, BuildHasherDefault<FxHasher>>,
    ) {
        // get the layer relative tile coordinates for the area that needs
        // to be set in this chunk
        let (this_layer_x0, this_layer_y0, this_layer_x1, this_layer_y1) =
            chunk_bounds.get_bound_coords(true);

        // loop over the layer tile coordinates
        for this_layer_y in this_layer_y0..this_layer_y1 {
            for this_layer_x in this_layer_x0..this_layer_x1 {
                // get the parent tile
                let parent_tile = parent_tiles[&(this_layer_x, this_layer_y)];
                // check existing tile
                let tile_value = child_layer
                    .get_at(this_layer_x, this_layer_y)
                    .unwrap_or(parent_tile);
                // set the tile vale in the chunk from the parent tile value
                let _ = child_layer.set_at(this_layer_x, this_layer_y, tile_value);
            }
        }
    }

    fn generate_chunk_work_from_parent(
        &self,
        parent_tiles: IndexMap<(isize, isize), TIndex, BuildHasherDefault<FxHasher>>,
        manager_guid_bytes: [u8; 16],
        child_layer_guid_bytes: [u8; 16],
        child_chunk_bounds: Bounds,
        child_tiles: IndexMap<(isize, isize), Option<TIndex>, BuildHasherDefault<FxHasher>>,
    ) -> IndexMap<(isize, isize), Option<TIndex>, BuildHasherDefault<FxHasher>> {
        todo!()
    }
}
