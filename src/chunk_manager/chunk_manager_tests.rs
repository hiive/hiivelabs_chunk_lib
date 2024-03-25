#[derive(Clone)]
struct TestMap {
    pub(crate) data: Vec<u8>,
    width: usize,
    height: usize,
}

impl TileMapDataSource<u8> for TestMap {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get_index_of(&self, x: usize, y: usize) -> Option<usize> {
        let ix = y * self.width + x;
        if ix < self.width * self.height {
            Some(ix)
        } else {
            None
        }
    }

    fn take_data(&mut self) -> Vec<u8> {
        // self.data.iter().collect()
        std::mem::take(&mut self.data)
    }
}

impl TestMap {
    pub fn new(width: usize, height: usize, is_random: bool) -> Self {
        let data = if is_random {
            Self::generate_random_vector(width * height)
        } else {
            (0..width * height)
                .map(|i| 0_u8.wrapping_add(i as u8))
                .collect()
        };
        Self {
            width,
            height,
            data,
        }
    }
    fn generate_random_vector(length: usize) -> Vec<u8> {
        let seed = [42; 32];
        let mut rng = StdRng::from_seed(seed);
        (0..length).map(|_| rng.gen()).collect()
    }
}

#[cfg(test)]
use super::ChunkManager;
use crate::TileMapDataSource;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub(crate) fn make_test_chunk_manager(
    width: usize,
    height: usize,
    layer_count: usize,
    chunk_width: usize,
    chunk_height: usize,
    chunk_padding_in_tiles: usize,
    use_random_map: bool,
) -> ChunkManager<u8> {
    let test_map = Box::new(TestMap::new(width, height, use_random_map));
    ChunkManager::<u8>::new(
        width,
        height,
        layer_count,
        chunk_width,
        chunk_height,
        chunk_padding_in_tiles,
        test_map,
    )
}

fn test_init_cm_with_params(
    width: usize,
    height: usize,
    layer_count: usize,
    chunk_width: usize,
    chunk_height: usize,
    chunk_padding_in_tiles: usize,
    use_random_map: bool,
) {
    let cm = make_test_chunk_manager(
        width,
        height,
        layer_count,
        chunk_width,
        chunk_height,
        chunk_padding_in_tiles,
        use_random_map,
    );
    assert_eq!(cm.width, width);
    assert_eq!(cm.height, height);

    for y in 0..height {
        for x in 0..width {
            let _cm_val = cm.get_at(x as isize, y as isize, 0);
            // let tm_val= comparison_test_map.get_at(x, y);
            // println!("{cm_val:?} :: {tm_val:?}");
            // assert_eq!(cm_val, tm_val);
        }
    }
}

#[test]
fn test_init() {
    test_init_cm_with_params(64, 32, 4, 32, 16, 1, true);
}
