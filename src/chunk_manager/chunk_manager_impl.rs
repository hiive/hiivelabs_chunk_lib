use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::tilemap_datasource::TileMapDataSource;
use crate::chunk_layer::{ChunkLayer, TIndex};

/// Manages a chunked 2D tilemap that automatically procedurally generates
/// additional procedural detail.
pub struct ChunkManager<T> {
    pub(crate) layers: Vec<Rc<RefCell<Option<ChunkLayer>>>>,
    /// The width in tiles of the top level map data.
    pub width: usize,
    /// The height in tiles of the top level map data.
    pub height: usize,
    pub(crate) owned_values: Vec<T>,
}

impl<T: std::fmt::Debug> ChunkManager<T> {
    /// Initializes a new instance of the `ChunkManager<T>`.
    ///
    /// # Arguments
    ///
    /// * `source`: The source data for the top level map, implementing the [`crate::TileMapDataSource<T>`] trait.
    /// * `layer_count`: The number of detail layers to procedurally generate.
    ///                  Each layer is four times the area of the previous layer.
    ///                  The `source` data is copied into layer `0`.
    /// * `layer_chunk_cache_size`: The number of procedurally generated chunks to keep in the cache.
    ///                  This is per layer. The recommended minimum value for most use case is `32`.
    /// * `chunk_width`: The width in tiles of the procedurally generated chunks.
    ///                  Each layer other than `0` is split into chunks.
    ///                  This must be exactly divisible by the `source` width.
    /// * `chunk_height`: The height in tiles of the procedurally generated chunks.
    ///                  Each layer other than `0` is split into chunks.
    ///                  This must be exactly divisible by the `source` height.
    /// * `chunk_padding_in_tiles`: The number of tiles by which to overlap chunks.
    ///                  The best value to use is `1`, as it minimizes edge artifacts
    ///                  with the current procedural generation method.
    ///
    /// Returns: `ChunkManager<T>` initialized with the `source` data.
    pub fn new(
        source: Box<dyn TileMapDataSource<T>>,
        layer_count: usize,
        layer_chunk_cache_size: u32,
        chunk_width: usize,
        chunk_height: usize,
        chunk_padding_in_tiles: usize,
    ) -> Self {

        let width = source.width();
        let height = source.height();
        assert!(
            width > 0 && height > 0,
            "width and height must both be greater than zero"
        );
        assert!(
            chunk_width > 0 && chunk_height > 0,
            "chunk_width and chunk_height must both be greater than zero"
        );
        assert!(
            width % chunk_width == 0 && height % chunk_height == 0,
            "width/height must be exactly divisible by chunk width/height"
        );
        assert!(layer_count > 0, "layer_count must be greater than zero");
        assert!(
            layer_chunk_cache_size > 0,
            "layer_chunk_cache_size must be greater than zero (Recommended > 32)"
        );

        // let's calculate a reasonable starting capacity for the owned_values vector.
        // For the top layer, we can expect the entire capacity to be needed.
        // for each subsequent layer, we can expect that 0.25 * ratio of layer area to previous layer
        // is needed. As each subsequent layer has 4 times the area of the previous layer, this
        // is 0.25 * 4 = 1.
        // so a good amount to reserve will be width * height * layer_count

        // create the owned values vector
        let mut owned_values = Vec::<T>::with_capacity(width * height * layer_count);

        #[cfg(debug_assertions)]
        println!("reserved space: {}", width * height * layer_count);

        // create the layers
        let layers = ChunkManager::init_layers(
            &mut owned_values,
            source,
            layer_count,
            layer_chunk_cache_size,
            chunk_width,
            chunk_height,
            chunk_padding_in_tiles,
        );

        Self {
            layers,
            width,
            height,
            owned_values,
        }
    }

    /// Retrieve the tile at coordinates `(x, y)` in layer `z`, where `z` is the
    /// zero-indexed layer number.
    /// If the specified tile does not exist in the layer, the chunk(s) containing that tile will
    /// be generated on demand. A generated chunk is guaranteed to be identical every time it is generated.
    ///
    /// Layer `0` has the `width` and `height` of the source data.
    /// Each subsequent layer is double the `width` and `height` of the source data.
    /// The `(x, y)` coordinates are relative to the requested layer `z`.
    ///
    /// # Arguments
    ///
    /// * `x`: The _x_ coordinate in the requested layer `z`.
    /// * `y`: The _y_ coordinate in the requested layer `z`.
    /// * `z`: The layer to query.
    ///
    /// Returns `Ok(&T)` if the specified coordinates are in bounds, else `Err(&str)`.
    pub fn get_at(&self, x: isize, y: isize, z: usize) -> Result<&T, &str> {
        let some_layer = self.layers.get(z);

        match some_layer {
            Some(layer_rc) => {
                let x = if z == 0 { x } else { x.pow(z as u32) };
                let y = if z == 0 { y } else { y.pow(z as u32) };
                // get the layer - this has to be in two stages to keep the ref
                // around long enough
                let layer_opt = layer_rc.borrow();
                let layer = layer_opt.as_ref().unwrap();
                // layer.get_at(x, y)
                match layer.get_at(x, y) {
                    Some(ix) => Ok(&self.owned_values[ix]),
                    _ => Err("(x, y) coordinates out of bounds"),
                }
            }
            _ => Err("Layer z coordinate out of bounds"),
        }
    }

    fn init_layers(
        owned_values: &mut Vec<T>,
        mut source: Box<dyn TileMapDataSource<T>>,
        layer_count: usize,
        layer_chunk_cache_size: u32,
        chunk_width: usize,
        chunk_height: usize,
        chunk_padding_in_tiles: usize,
    ) -> Vec<Rc<RefCell<Option<ChunkLayer>>>> {
        let width_in_chunks = source.width() / chunk_width;
        let height_in_chunks = source.height() / chunk_height;

        // create the layers
        let mut layers = Vec::with_capacity(layer_count);
        let mut prev_layer = Rc::new(RefCell::new(None));
        for layer_id in 0..layer_count {
            // each layer is double the width/height of the previous one.
            let f = 2_usize.pow((layer_id + 1) as u32);
            let mut current_layer = ChunkLayer::new(
                Rc::clone(&prev_layer),
                layer_id,
                layer_chunk_cache_size,
                f * width_in_chunks,
                f * height_in_chunks,
                chunk_padding_in_tiles,
                chunk_width,
                chunk_height,
            );

            // if we're on the top layer, populate it with the map data
            if layer_id == 0 {
                // build the map of vec index to (x, y) to drain the vector
                Self::populate_top_layer_from_source(owned_values, &mut source, &mut current_layer);
            }
            // set up prev layer for the next iteration
            prev_layer = ChunkLayer::make_layer_rc(Some(current_layer));
            // prev layer actually contains the current layer at this point,
            // so add it to the layer vector
            layers.push(Rc::clone(&prev_layer));
        }

        layers
    }

    fn populate_top_layer_from_source(
        owned_values: &mut Vec<T>,
        mut source: &mut Box<dyn TileMapDataSource<T>>,
        mut current_layer: &mut ChunkLayer,
    ) {
        let width = source.width();
        let height = source.height();

        let mut ix_map = HashMap::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                if let Some(ix) = source.get_index_of(x, y) {
                    ix_map.insert(ix, (x as isize, y as isize));
                }
            }
        }

        // populate top layer
        let mut map_data = { source.take_data() }; // take the map data
        let top_layer = &mut current_layer;
        for (ix, item) in map_data.drain(..).enumerate() {
            let (x, y) = ix_map[&ix];
            // store the value and set the layer coords with the index value
            // for the stored value
            owned_values.push(item);
            {
                let t_index: TIndex = owned_values.len() - 1;
                top_layer.set_at(x, y, t_index);
            }
        }
    }
}
