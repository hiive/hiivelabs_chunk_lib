# HiiveLabs Chunk Library

A Rust library for efficient chunk-based world generation and management, primarily used for game development and
procedural content generation.

## Features

- **Chunk Management**: Efficient loading, unloading, and caching of world chunks
- **Tile Map Data Source**: Interface for procedural generation of tile-based content
- **Storage Integration**: Built-in support for persistent chunk storage
- **Random Utilities**: Integration with random number generation for procedural content
- **Performance Optimized**: Uses caching and compression for optimal memory usage

## Dependencies

- `hiivelabs_storage_lib` - Storage and persistence functionality
- `hiivelabs_rand_utils_lib` - Random utilities and procedural generation helpers
- `smallvec` - Stack-allocated vectors for performance
- `schnellru` - LRU cache implementation
- `bitcode` - Binary serialization
- `miniz_oxide` - Compression support
- `uuid` - Unique identifiers for chunks
- `blake2` - Hashing for chunk verification
- `indexmap` - Ordered hash maps
- `rustc-hash` - Fast hash implementation

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
hiivelabs_chunk_lib = { version = "0.1.0", path = "path/to/hiivelabs_chunk_lib" }
```

### Basic Example

```rust
use hiivelabs_chunk_lib::prelude::*;

// Initialize chunk manager
let chunk_manager = ChunkManager::new();

// Create a tile map data source for procedural generation
let data_source = TileMapDataSource::new();

// Use the chunk manager to load and manage world chunks
```

## Architecture

The library is organized into several key modules:

- **chunk_manager**: Core chunk loading and management system
- **tilemap_datasource**: Interface for procedural tile generation
- **chunk**: Individual chunk data structures
- **chunk_layer**: Layer-based chunk organization
- **chunk_storage**: Persistent storage functionality
- **chunk_generator**: Procedural chunk generation
- **chunk_seed_utils**: Utilities for deterministic generation

## Version

Current version: 0.1.0

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

This project uses the Rust 2021 edition.