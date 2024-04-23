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
        child_chunk_bounds: Bounds,
        child_layer: &mut ChunkLayer,
        parent_tiles: IndexMap<(isize, isize), TIndex, BuildHasherDefault<FxHasher>>,
    ) {
        let (this_layer_x0, this_layer_y0, this_layer_x1, this_layer_y1) =
            child_chunk_bounds.get_bound_coords(true);

        let w = 2 + this_layer_x1 - this_layer_x0;
        let h = 2 + this_layer_y1 - this_layer_y0;
        let s = (w * h) as usize;
        let mut child_tiles: IndexMap<
            (isize, isize),
            Option<TIndex>,
            BuildHasherDefault<FxHasher>,
        > = IndexMap::with_capacity_and_hasher(s, BuildHasherDefault::default());

        // build the child tile map
        for this_layer_y in this_layer_y0..this_layer_y1 {
            for this_layer_x in this_layer_x0..this_layer_x1 {
                // check existing tile
                let tile_value = child_layer.get_at(this_layer_x, this_layer_y);
                child_tiles.insert((this_layer_x, this_layer_y), tile_value);
            }
        }

        child_tiles = self.generate_chunk_work_from_parent(
            parent_tiles,
            child_layer.manager_guid_bytes,
            child_layer.layer_guid_bytes,
            child_chunk_bounds,
            child_tiles,
        );

        assert!(child_tiles.len() <= s);

        // now fill the chunk from the tiles
        for ((tx, ty), tile_value) in child_tiles.drain(..) {
            let _ = child_layer.set_at(tx, ty, tile_value.expect("tile value not set"));
        }
    }

    fn generate_chunk_work_from_parent(
        &self,
        parent_tiles: IndexMap<(isize, isize), TIndex, BuildHasherDefault<FxHasher>>,
        _manager_guid_bytes: [u8; 16],     /* unused */
        _child_layer_guid_bytes: [u8; 16], /* unused */
        child_chunk_bounds: Bounds,
        mut child_tiles: IndexMap<(isize, isize), Option<TIndex>, BuildHasherDefault<FxHasher>>,
    ) -> IndexMap<(isize, isize), Option<TIndex>, BuildHasherDefault<FxHasher>> {
        let (this_layer_x0, this_layer_y0, this_layer_x1, this_layer_y1) =
            child_chunk_bounds.get_bound_coords(true);

        // first pass - just double the chunks
        for this_layer_y in this_layer_y0..this_layer_y1 {
            for this_layer_x in this_layer_x0..this_layer_x1 {
                // get the parent tile
                let parent_tile = parent_tiles[&(this_layer_x, this_layer_y)];
                // check existing tile
                let tile_value = child_tiles[&(this_layer_x, this_layer_y)].unwrap_or(parent_tile);
                // set the tile vale in the chunk from the parent tile value
                child_tiles.insert((this_layer_x, this_layer_y), Some(tile_value));
            }
        }
        child_tiles
    }
}
