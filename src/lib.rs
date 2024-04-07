#![allow(dead_code)]

mod random;
mod bounds;
mod chunk;
mod chunk_layer;
pub mod chunk_manager;
pub mod tilemap_datasource;

#[cfg(test)]
mod test_utils;

pub mod prelude {
    // exports
    pub use crate::chunk_manager::ChunkManager;
    pub use crate::tilemap_datasource::TileMapDataSource;
}
