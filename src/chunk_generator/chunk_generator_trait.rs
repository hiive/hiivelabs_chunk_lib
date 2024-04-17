use crate::bounds::Bounds;
use crate::chunk_layer::{ChunkLayer, TIndex};
use indexmap::IndexMap;

pub(crate) trait ChunkGenerator {
    fn generate_chunk_from_parent(
        &self,
        chunk_bounds: Bounds,
        child_layer: &mut ChunkLayer,
        parent_tiles: IndexMap<(isize, isize), TIndex>,
    );
}
