use crate::chunk_layer::ChunkLayer;
use crate::tilemap_datasource::TileMapDataSource;
pub struct ChunkManager<'m, T> {
    layers: Vec<ChunkLayer<'m, T>>,
    width: usize,
    height: usize,
}

impl<'m, T: std::fmt::Debug> ChunkManager<'m, T> {
    pub fn new(
        width: usize,
        height: usize,
        layer_count: usize,
        chunk_width: usize,
        chunk_height: usize,
        chunk_padding_in_tiles: usize,
    ) -> Self {
        assert!(
            width % chunk_width == 0 && height % chunk_height == 0,
            "width/height must be exactly divisible by chunk width/height"
        );

        let mut layers = Vec::with_capacity(layer_count);
        // initialize the top layer
        let top_layer =
            ChunkLayer::<T>::new(0, 1, 1, chunk_padding_in_tiles, width, height, None);
        layers.push(top_layer);

        let width_in_chunks = width / chunk_width;
        let height_in_chunks = height / chunk_height;

        for layer_id in 1..layer_count {
            let f = 2usize.pow(layer_id as u32);
            let prev_layer = layers.last();
            let layer = ChunkLayer::<T>::new(
                layer_id,
                f * width_in_chunks,
                f * height_in_chunks,
                chunk_padding_in_tiles,
                chunk_width,
                chunk_height,
                prev_layer
            );

        }

        Self {
            layers,
            width,
            height,
        }
    }

    pub fn init_top_layer<S: TileMapDataSource<T>>(&mut self, source: &'m S) {
        let w = source.width();
        let h = source.height();
        assert!(
            w == self.width && h == self.height,
            "source is of wrong dimensions."
        );

        // copy data
        let data = source.get_data();
        let mut ix = 0;
        let layer0 = self.layers.get_mut(0).unwrap();
        for y in 0..h {
            for x in 0..w {
                layer0.set_at(x as isize, y as isize, data[ix]);
                ix += 1;
            }
        }
    }

    pub fn get_at(&self, x: isize, y: isize, z:usize) -> Option<&'m T> {
        let some_layer = self.layers.get(z);

        match some_layer {
            Some(layer) => {
                let x = if z == 0 { x } else { x.pow(z as u32) };
                let y= if z == 0 { y } else { y.pow(z as u32) };
                layer.get_at(x, y)
            },
            _ => None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
// tests
////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
mod chunk_manager_tests {


    struct TestMap {
        pub(crate) data: Vec<u8>,
        width: usize,
        height:usize,
    }

    impl TileMapDataSource<u8> for TestMap {
        fn width(&self) -> usize {
            self.width
        }

        fn height(&self) -> usize {
            self.height
        }

        fn get_at(&self, x: usize, y: usize) -> Option<&u8> {
            let ix = y * self.width + x;
            self.data.get(ix)
        }

        fn get_data(&self) -> Vec<&u8> {
            self.data.iter().collect()
        }
    }

    impl TestMap {

        pub fn new(width: usize, height: usize, is_random:bool) -> Self {

            let data = if is_random {
                Self::generate_random_vector(width * height)
            }
            else{
                (0..width*height)
                    .map(|i| 0_u8.wrapping_add(i as u8))
                    .collect()
            };
            Self{
                width,
                height,
                data
            }
        }
        fn generate_random_vector(length: usize) -> Vec<u8> {
            let seed = [42; 32];
            let mut rng = StdRng::from_seed(seed);
            (0..length).map( | _| rng.gen()).collect()
        }
    }


    #[cfg(test)]
    use std::fmt::Debug;
    use super::ChunkManager;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use crate::TileMapDataSource;


    fn test_init_with_params(width:usize, height:usize, layer_count:usize,
                             chunk_width:usize, chunk_height:usize,
                             chunk_padding_in_tiles:usize,
                             use_random_map:bool) {

        let test_map = TestMap::new(width, height, use_random_map);

        let mut cm = ChunkManager::<u8>::new(width, height, layer_count,
                                             chunk_width, chunk_height, chunk_padding_in_tiles);

        assert_eq!(cm.width, width);
        assert_eq!(cm.height, height);

        cm.init_top_layer(&test_map);

        for y in 0..height {
            for x in 0..width {
                let cm_val = cm.get_at(x as isize, y as isize, 0);
                let tm_val= test_map.get_at(x, y);
                // println!("{cm_val:?} :: {tm_val:?}");
                assert_eq!(cm_val, tm_val);
            }
        }
    }

    #[test]
    fn test_init()
    {

        test_init_with_params(64, 32, 4,
                              32, 16, 1,
                              true);
    }
}
