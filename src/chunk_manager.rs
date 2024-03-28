use crate::TileMapDataSource;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::chunk_layer::{ChunkLayer, TIndex};
pub struct ChunkManager<T> {
    layers: Vec<Rc<RefCell<Option<ChunkLayer>>>>,
    width: usize,
    height: usize,
    owned_values: Vec<T>,
}

impl<'l, T: std::fmt::Debug> ChunkManager<T> {
    pub fn new(
        width: usize,
        height: usize,
        layer_count: usize,
        chunk_width: usize,
        chunk_height: usize,
        chunk_padding_in_tiles: usize,
        source: Box<dyn TileMapDataSource<T>>,
    ) -> Self {
        assert!(
            width % chunk_width == 0 && height % chunk_height == 0,
            "width/height must be exactly divisible by chunk width/height"
        );

        // create the owned values vector
        let mut owned_values = Vec::<T>::with_capacity(width * height); // todo - size calc correctly
                                                                        // create the layers
        let layers = ChunkManager::init_layers(
            &mut owned_values,
            source,
            layer_count,
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

    fn init_layers(
        mut owned_values: &mut Vec<T>,
        mut source: Box<dyn TileMapDataSource<T>>,
        layer_count: usize,
        chunk_width: usize,
        chunk_height: usize,
        chunk_padding_in_tiles: usize,
    ) -> Vec<Rc<RefCell<Option<ChunkLayer>>>> {
        let height = source.height();
        let width = source.width();
        let width_in_chunks = width / chunk_width;
        let height_in_chunks = height / chunk_height;

        // create the layers
        let mut layers = Vec::with_capacity(layer_count);
        let mut prev_layer: Rc<RefCell<Option<ChunkLayer>>> = Rc::new(RefCell::new(None));
        for layer_id in 0..layer_count {
            let f = 2usize.pow((layer_id + 1) as u32);
            let parent_layer = Rc::clone(&prev_layer);
            let mut current_layer = ChunkLayer::new(
                parent_layer,
                layer_id,
                f * width_in_chunks,
                f * height_in_chunks,
                chunk_padding_in_tiles,
                chunk_width,
                chunk_height,
            );

            // build the map of vec index to (x, y) to drain the vector
            let mut ix_map = HashMap::new();
            for y in 0..height {
                for x in 0..width {
                    if let Some(ix) = source.get_index_of(x, y) {
                        ix_map.insert(ix, (x as isize, y as isize));
                    }
                }
            }

            // if we're on the top layer, populate it with the map data
            if layer_id == 0 {
                // populate top layer
                let mut map_data = { source.take_data() }; // take the map data
                let top_layer = &mut current_layer;
                for (ix, item) in map_data.drain(..).enumerate() {
                    let (x, y) = ix_map[&ix];
                    // store the value and set the layer coords with the index value
                    // for the stored value
                    owned_values.push(item);
                    {
                        top_layer.set_at(x, y, owned_values.len() -1);
                    }
                }
            }
            // set up prev layer for the next iteration
            prev_layer = ChunkLayer::make_layer_rc(Some(current_layer));
            // prev layer actually contains the current layer at this point,
            // so add it to the layer vector
            layers.push(Rc::clone(&prev_layer));
        }

        layers
    }

    pub fn get_at(&self, x: isize, y: isize, z: usize) -> Option<&T> {
        let some_layer = self.layers.get(z);

        match some_layer {
            Some(layer_rc) => {
                let x = if z == 0 { x } else { x.pow(z as u32) };
                let y = if z == 0 { y } else { y.pow(z as u32) };
                // get the layer - this has to be in two stages to keep the ref
                // around long enough
                let layer_ref = layer_rc.borrow();
                let layer = layer_ref.as_ref().unwrap();
                // layer.get_at(x, y)
                match layer.get_at(x, y) {
                    Some(ix) => Some(&self.owned_values[ix]),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
// tests
////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) mod chunk_manager_tests;
