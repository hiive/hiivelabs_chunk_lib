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
            ChunkLayer::<T>::new(0, 1, 1, chunk_padding_in_tiles, chunk_width, chunk_height);
        layers.push(top_layer);

        let width_in_chunks = width / chunk_width;
        let height_in_chunks = height / chunk_height;

        for layer_id in 1..layer_count {
            let f = 2usize.pow(layer_id as u32);
            let layer = ChunkLayer::<T>::new(
                layer_id,
                f * width_in_chunks,
                f * height_in_chunks,
                chunk_padding_in_tiles,
                chunk_width,
                chunk_height,
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
// tests
////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
mod chunk_manager_tests {}
