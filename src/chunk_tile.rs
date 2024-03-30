
use crate::chunk_layer::TIndex;

use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct ChunkTile {
    #[cfg(debug_assertions)]
    pub x: isize,
    #[cfg(debug_assertions)]
    pub y: isize,
    pub value: TIndex,
}

impl ChunkTile {
    #[cfg(debug_assertions)]
    pub(crate) fn new(x: isize, y: isize, value: usize) -> Self {
        Self { x, y, value }
    }

    #[cfg(not(debug_assertions))]
    pub(crate) fn new(value: usize) -> Self {
        Self { value }
    }
}
