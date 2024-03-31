use crate::ChunkManager;
use crate::TileMapDataSource;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Instant;

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

    fn get_default_out_of_bounds_value_index(&self) -> usize {
        0xFF
    }
}

impl TestMap {
    pub fn new(width: usize, height: usize, is_random: bool) -> Self {
        let data = if is_random {
            Self::generate_random_vector(width * height)
        } else {
            (0..width * height)
                .map(|i| 0_u8.wrapping_add(i as u8) % 0xFE)
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

pub(crate) fn make_test_chunk_manager(
    width: usize,
    height: usize,
    layer_count: usize,
    layer_chunk_lru_cache_size: u32,
    chunk_width: usize,
    chunk_height: usize,
    chunk_padding_in_tiles: usize,
    use_random_map: bool,
) -> ChunkManager<u8> {
    let start = Instant::now(); // Start timing
    let test_map = Box::new(TestMap::new(width, height, use_random_map));
    let duration = start.elapsed();
    println!("Random map creation in {duration:?}");

    let start = Instant::now(); // Start timing
    let cm = ChunkManager::<u8>::new(
        test_map,
        layer_count,
        layer_chunk_lru_cache_size,
        chunk_width,
        chunk_height,
        chunk_padding_in_tiles,
    );

    let duration = start.elapsed();
    println!("Chunk Manager initialization in {duration:?}");
    println!();
    cm
}

fn test_init_cm_with_params(
    width: usize,
    height: usize,
    layer_count: usize,
    layer_chunk_lru_cache_size: u32,
    chunk_width: usize,
    chunk_height: usize,
    chunk_padding_in_tiles: usize,
    use_random_map: bool,
) {
    let cm = make_test_chunk_manager(
        width,
        height,
        layer_count,
        layer_chunk_lru_cache_size,
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
fn test_small_init() {
    test_init_cm_with_params(64, 32, 4, 32, 32, 16, 1, true);
}

#[test]
fn test_large_init() {
    test_init_cm_with_params(1024, 768, 8, 32, 32, 16, 1, true);
}

#[test]
#[should_panic(expected = "layer_chunk_cache_size must be greater than zero (Recommended > 32)")]
fn test_zero_lru_size() {
    let cm = make_test_chunk_manager(10, 10, 1, 0, 5, 5, 0, false);
}

#[test]
#[should_panic(expected = "width and height must both be greater than zero")]
fn test_zero_width() {
    let cm = make_test_chunk_manager(0, 10, 1, 0, 5, 5, 0, false);
}

#[test]
#[should_panic(expected = "width and height must both be greater than zero")]
fn test_zero_height() {
    let cm = make_test_chunk_manager(10, 0, 1, 0, 5, 5, 0, false);
}

#[test]
#[should_panic(expected = "width and height must both be greater than zero")]
fn test_zero_width_and_height() {
    let cm = make_test_chunk_manager(0, 0, 1, 0, 5, 5, 0, false);
}

#[test]
#[should_panic(expected = "chunk_width and chunk_height must both be greater than zero")]
fn test_zero_chunk_width() {
    let cm = make_test_chunk_manager(10, 10, 1, 32, 0, 5, 0, false);
}

#[test]
#[should_panic(expected = "chunk_width and chunk_height must both be greater than zero")]
fn test_zero_chunk_height() {
    let cm = make_test_chunk_manager(10, 20, 1, 32, 5, 0, 0, false);
}

#[test]
#[should_panic(expected = "layer_count must be greater than zero")]
fn test_zero_layer_count_height() {
    let cm = make_test_chunk_manager(10, 10, 0, 32, 5, 5, 0, false);
}

#[test]
fn test_can_get_from_non_zero_layer() {
    let cm = make_test_chunk_manager(8, 8, 4, 32, 8, 8, 1, true);

    cm.print_debug_layers();

    let c = 2_isize.pow(4);
    println!("Looking at ({c}, {c}, 3)");
    let test_val = cm.get_at(c, c, 3).unwrap();
    println!("test_val: {test_val:02X}");

    cm.print_debug_layers();

    for y in 0..cm.height as isize * 2 {
        for x in 0..cm.width as isize * 2 {
            let v1 = cm.get_at(x, y, 1);
            let v0 = cm.get_at(x / 2, y / 2, 0);
            assert_eq!(v1, v0);
        }
    }


}
