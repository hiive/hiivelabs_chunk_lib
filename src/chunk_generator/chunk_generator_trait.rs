use std::hash::BuildHasherDefault;
use crate::bounds::Bounds;
use crate::chunk_layer::{ChunkLayer, TIndex};
use indexmap::IndexMap;
use rustc_hash::FxHasher;

pub(crate) trait ChunkGenerator {
    fn generate_chunk_from_parent(
        &self,
        chunk_bounds: Bounds,
        child_layer: &mut ChunkLayer,
        parent_tiles: IndexMap<(isize, isize), TIndex, BuildHasherDefault<FxHasher>>,
    );
}
