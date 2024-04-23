use crate::bounds::Bounds;
use crate::chunk_generator::chunk_generator_trait::ChunkGenerator;
use crate::chunk_layer::{ChunkLayer, TIndex};
use crate::chunk_seed_utils::chunk_seed_utils_impl::create_seed_from_guid_bytes_x_y;
use indexmap::IndexMap;
use rustc_hash::FxHasher;
use std::hash::{BuildHasherDefault, Hasher};

pub(crate) struct ChunkSeededInterpolator;

impl ChunkGenerator for ChunkSeededInterpolator {
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

        // TODO - get thread work result.
        // now fill the chunk from the tiles
        for ((tx, ty), tile_value) in child_tiles.drain(..) {
            let _ = child_layer.set_at(tx, ty, tile_value.expect("tile value not set"));
        }
    }

    fn generate_chunk_work_from_parent(
        &self,
        parent_tiles: IndexMap<(isize, isize), TIndex, BuildHasherDefault<FxHasher>>,
        manager_guid_bytes: [u8; 16],
        child_layer_guid_bytes: [u8; 16],
        child_chunk_bounds: Bounds,
        mut child_tiles: IndexMap<(isize, isize), Option<TIndex>, BuildHasherDefault<FxHasher>>,
    ) -> IndexMap<(isize, isize), Option<TIndex>, BuildHasherDefault<FxHasher>> {
        // get the layer relative tile coordinates for the area that needs
        // to be set in this chunk
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

        // let's get the random seed.
        let seed = {
            let mut result = [0u8; 32];
            let seed2 = create_seed_from_guid_bytes_x_y(
                &child_layer_guid_bytes,
                this_layer_x0,
                this_layer_y0,
            );
            let seed1 = manager_guid_bytes;
            result[..16].copy_from_slice(&seed1);
            result[16..].copy_from_slice(&seed2);
            result
        };
        let mut hasher = FxHasher::default();

        // second pass - copy some chunks from their neighbors
        for this_layer_y in (this_layer_y0 - 1..this_layer_y1 + 1).step_by(2) {
            for this_layer_x in this_layer_x0 - 1..this_layer_x1 + 1
            /*.step_by(2)*/
            {
                Self::choose_child_tile(
                    &mut child_tiles,
                    &mut hasher,
                    &seed,
                    this_layer_y,
                    this_layer_x,
                );
            }
        }

        // third pass - copy some chunks from their neighbors
        for this_layer_y in this_layer_y0 - 1..this_layer_y1 + 1
        /*.step_by(2)*/
        {
            for this_layer_x in (this_layer_x0 - 1..this_layer_x1 + 1).step_by(2) {
                Self::choose_child_tile(
                    &mut child_tiles,
                    &mut hasher,
                    &seed,
                    this_layer_y,
                    this_layer_x,
                );
            }
        }
        child_tiles
    }
}

impl ChunkSeededInterpolator {
    #[inline(always)]
    fn choose_child_tile(
        child_tiles: &mut IndexMap<(isize, isize), Option<TIndex>, BuildHasherDefault<FxHasher>>,
        hasher: &mut FxHasher,
        seed: &[u8; 32],
        this_layer_y: isize,
        this_layer_x: isize,
    ) {
        let dx = Self::get_coord_offset(hasher, seed, this_layer_x);
        let dy = Self::get_coord_offset(hasher, seed, this_layer_y);
        if let Some(Some(tile_value)) = child_tiles.get(&(this_layer_x + dx, this_layer_y + dy)) {
            child_tiles.insert((this_layer_x, this_layer_y), Some(*tile_value));
        }
    }

    #[inline(always)]
    fn get_coord_offset(hasher: &mut FxHasher, seed: &[u8; 32], coord: isize) -> isize {
        hasher.write(seed);
        hasher.write_isize(coord);
        (hasher.finish() % 3) as isize - 1
    }
}
