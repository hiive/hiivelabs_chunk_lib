use crate::bounds::Bounds;
use crate::chunk_layer::{ChunkLayer, TIndex};
use indexmap::IndexMap;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

pub(crate) trait ChunkGenerator {
    fn generate_chunk_from_parent(
        &self,
        chunk_bounds: Bounds,
        child_layer: &mut ChunkLayer,
        parent_tiles: IndexMap<(isize, isize), TIndex, BuildHasherDefault<FxHasher>>,
    );

    fn generate_chunk_work_from_parent(
        &self,
        parent_tiles: IndexMap<(isize, isize), TIndex, BuildHasherDefault<FxHasher>>,
        manager_guid_bytes: [u8; 16],
        child_layer_guid_bytes: [u8; 16],
        child_chunk_bounds: Bounds,
        /* mut */ child_tiles: IndexMap<(isize, isize), Option<TIndex>, BuildHasherDefault<FxHasher>>,
    ) -> IndexMap<(isize, isize), Option<TIndex>, BuildHasherDefault<FxHasher>>;
}
