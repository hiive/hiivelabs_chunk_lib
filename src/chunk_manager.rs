use crate::TileMapDataSource;
use std::collections::HashMap;

use crate::chunk_layer::ChunkLayer;
pub struct ChunkManager<T> {
    layers: Vec<ChunkLayer<T>>,
    width: usize,
    height: usize,
}

impl<T: std::fmt::Debug> ChunkManager<T> {
    pub fn new(
        width: usize,
        height: usize,
        layer_count: usize,
        chunk_width: usize,
        chunk_height: usize,
        chunk_padding_in_tiles: usize,
        mut source: Box<dyn TileMapDataSource<T>>,
    ) -> Self {
        assert!(
            width % chunk_width == 0 && height % chunk_height == 0,
            "width/height must be exactly divisible by chunk width/height"
        );

        // create the layers
        let mut cm = Self {
            layers: ChunkManager::init_layers(
                source,
                layer_count,
                chunk_width,
                chunk_height,
                chunk_padding_in_tiles,
            ),
            width,
            height,
        };

        // let acm = ;
        /*
        let cm_layers = &mut cm.layers;
        for layer in cm_layers {
            layer.set_manager(&Arc::new(&cm));
        }
         */

        cm
    }

    fn init_layers(
        mut source: Box<dyn TileMapDataSource<T>>,
        layer_count: usize,
        chunk_width: usize,
        chunk_height: usize,
        chunk_padding_in_tiles: usize,
    ) -> Vec<ChunkLayer<T>> {
        let height = source.height();
        let width = source.width();
        let width_in_chunks = width / chunk_width;
        let height_in_chunks = height / chunk_height;
        // build the map to drain the vector
        let mut ix_map = HashMap::new();
        for y in 0..height {
            for x in 0..width {
                if let Some(ix) = source.get_index_of(x, y) {
                    ix_map.insert(ix, (x as isize, y as isize));
                }
            }
        }

        let mut layers = Vec::with_capacity(layer_count);
        for layer_id in 1..=layer_count {
            let f = 2usize.pow(layer_id as u32);
            layers.push(ChunkLayer::<T>::new(
                // &cm,
                layer_id,
                f * width_in_chunks,
                f * height_in_chunks,
                chunk_padding_in_tiles,
                chunk_width,
                chunk_height,
            ))
        }

        let top_layer = &mut layers[0];
        // take ownership of the map data
        let mut map_data = { source.take_data() };

        // populate the top layer

        // let top_layer = layers.get_mut(0);
        for (ix, item) in map_data.drain(..).enumerate() {
            let (x, y) = ix_map[&ix];
            {
                top_layer.set_at(x, y, item);
            }
        }

        layers
    }

    pub fn get_at(&self, x: isize, y: isize, z: usize) -> Option<&T> {
        let some_layer = self.layers.get(z);

        match some_layer {
            Some(layer) => {
                let x = if z == 0 { x } else { x.pow(z as u32) };
                let y = if z == 0 { y } else { y.pow(z as u32) };
                layer.get_at(x, y)
            }
            _ => None,
        }
    }

    fn ensure_layer(&mut self, x: isize, y: isize, z: usize)
    {
        // let layers = &mut self.layers;
        // for layer_ix in 1..=z {
        //     let prev_layer = layers.get(layer_ix - 1)
        //         .expect("Previous Layer missing");
        //     let some_layer = layers.get_mut(layer_ix);
        //
        //     match some_layer {
        //         Some(target_layer) => {
        //             let x = if z == 0 { x } else { x.pow(layer_ix as u32) };
        //             let y = if z == 0 { y } else { y.pow(layer_ix as u32) };
        //             match target_layer.get_at(x, y) {
        //                 Some(_) => {}
        //                 None => {
        //
        //                     target_layer.populate_chunk_at(prev_layer, x, y);
        //                 }
        //
        //             }
        //
        //         }
        //         None => {}
        //     }
        // }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
// tests
////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) mod chunk_manager_tests;
