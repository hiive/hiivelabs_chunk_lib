use crate::prelude::ChunkManager;
use crate::prelude::TileMapDataSource;

use hiivelabs_rand_utils_lib::utils::test_utils::*;
use hiivelabs_storage_lib::prelude::UniqueId;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fmt::Formatter;
use std::time::Instant;
use std::{fmt, fs};
use uuid::Uuid;

/// A wrapper around `u8` that implements `Debug` to display the value in hexadecimal.
#[derive(Clone)]
pub(crate) struct HexU8(u8);

/// Implement `Debug` for `HexU8` to format the inner `u8` value as hexadecimal.
impl fmt::Debug for HexU8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let color = if self.0 < 128 {
            _CONSOLE_BLUE
        } else if self.0 < 255 {
            _CONSOLE_GREEN
        } else {
            _CONSOLE_BRIGHT_GREEN
        };
        write!(f, "{color}{:02x}{_CONSOLE_DEFAULT_COLOR}", self.0)
    }
}

impl fmt::UpperHex for HexU8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let color = if self.0 < 128 {
            _CONSOLE_BRIGHT_BLUE
        } else if self.0 < 255 {
            _CONSOLE_BRIGHT_GREEN
        } else {
            _CONSOLE_BRIGHT_GREEN
        };
        write!(f, "{color}{:02X}{_CONSOLE_DEFAULT_COLOR}", self.0)
    }
}

#[derive(Clone)]
struct TestMap {
    pub(crate) data: Vec<HexU8>,
    width: usize,
    height: usize,
    oob_index: usize,
}

impl TileMapDataSource<HexU8> for TestMap {
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

    fn take_data(&mut self) -> Vec<HexU8> {
        // self.data.iter().collect()
        std::mem::take(&mut self.data)
    }

    fn get_default_out_of_bounds_value_index(&self) -> usize {
        self.oob_index
    }
}

impl TestMap {
    pub fn new(width: usize, height: usize, is_random: bool) -> Self {
        let data_len = width * height;
        let mut data = if is_random {
            Self::generate_random_vector(data_len)
        } else {
            (0..data_len)
                .map(|i| HexU8(0_u8.wrapping_add(i as u8) % 0xFE))
                .collect()
        };
        // ensure that there is a 0xFF in the data
        let oob_index = {
            if data_len > 0 {
                let oob_index = data_len - 1;
                data[oob_index] = HexU8(0xFF);
                log::info!("OOB VALUE: {:?}", data[oob_index]);
                oob_index
            } else {
                0
            }
        };

        Self {
            width,
            height,
            data,
            oob_index,
        }
    }
    fn generate_random_vector(length: usize) -> Vec<HexU8> {
        let seed = [42; 32];
        let mut rng = StdRng::from_seed(seed);
        (0..length).map(|_| HexU8(rng.gen())).collect()
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
    guid: Option<Uuid>,
) -> ChunkManager<HexU8> {
    let start = Instant::now(); // Start timing
    let test_map = Box::new(TestMap::new(width, height, use_random_map));
    let duration = start.elapsed();
    log::info!("Random map creation in {duration:?}");

    let start = Instant::now(); // Start timing
    let cm = ChunkManager::<HexU8>::new(
        test_map,
        layer_count,
        layer_chunk_lru_cache_size,
        chunk_width,
        chunk_height,
        chunk_padding_in_tiles,
        guid,
    );

    let duration = start.elapsed();
    log::info!("Chunk Manager initialization in {duration:?}");
    log::info!("");
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
    last_guid_byte: u8,
) {
    let guid_str = &format!("00000000-0000-0000-0000-0000000000{last_guid_byte:02x}");
    // clean up db
    let _ = fs::remove_file(format!("{guid_str}.world"));

    let guid = Some(Uuid::parse_str(guid_str).unwrap());
    let cm = make_test_chunk_manager(
        width,
        height,
        layer_count,
        layer_chunk_lru_cache_size,
        chunk_width,
        chunk_height,
        chunk_padding_in_tiles,
        use_random_map,
        guid,
    );
    assert_eq!(cm.width, width);
    assert_eq!(cm.height, height);

    for y in 0..height {
        for x in 0..width {
            let _cm_val = cm.get_at(x as isize, y as isize, 0);
            // let tm_val= comparison_test_map.get_at(x, y);
            // log::info!("{cm_val:?} :: {tm_val:?}");
            // assert_eq!(cm_val, tm_val);
        }
    }
}

#[test]
fn test_small_init() {
    setup_test_logger();
    test_init_cm_with_params(64, 32, 4, 32, 32, 16, 1, true, 0xFF);
}

#[test]
fn test_large_init() {
    setup_test_logger();
    test_init_cm_with_params(1024, 768, 8, 32, 32, 16, 1, true, 0x00);
}

#[test]
#[should_panic(expected = "layer_chunk_cache_size must be greater than zero (Recommended > 32)")]
fn test_zero_lru_size() {
    setup_test_logger();
    let _cm = make_test_chunk_manager(12, 12, 1, 0, 6, 6, 0, false, None);
}

#[test]
#[should_panic(expected = "width and height must both be greater than zero")]
fn test_zero_width() {
    setup_test_logger();
    let _cm = make_test_chunk_manager(0, 10, 1, 0, 5, 5, 0, false, None);
}

#[test]
#[should_panic(expected = "width and height must both be greater than zero")]
fn test_zero_height() {
    setup_test_logger();
    let _cm = make_test_chunk_manager(10, 0, 1, 0, 5, 5, 0, false, None);
}

#[test]
#[should_panic(expected = "width and height must both be greater than zero")]
fn test_zero_width_and_height() {
    setup_test_logger();
    let _cm = make_test_chunk_manager(0, 0, 1, 0, 5, 5, 0, false, None);
}

#[test]
#[should_panic(expected = "chunk_width and chunk_height must both be greater than zero")]
fn test_zero_chunk_width() {
    setup_test_logger();
    let _cm = make_test_chunk_manager(10, 10, 1, 32, 0, 5, 0, false, None);
}

#[test]
#[should_panic(expected = "chunk_width and chunk_height must both be greater than zero")]
fn test_zero_chunk_height() {
    setup_test_logger();
    let _cm = make_test_chunk_manager(10, 20, 1, 32, 6, 0, 0, false, None);
}

#[test]
#[should_panic(expected = "layer_count must be greater than zero")]
fn test_zero_layer_count_height() {
    setup_test_logger();
    let _cm = make_test_chunk_manager(12, 12, 0, 32, 6, 6, 0, false, None);
}

#[test]
fn test_unique_id() {
    setup_test_logger();
    let guid = Some(Uuid::parse_str("00000000-0000-0000-0000-222222222222").unwrap());
    let cm = make_test_chunk_manager(8, 8, 4, 1024, 8, 8, 1, true, guid);
    let uid = cm.get_unique_id(true);
    log::info!("Unique Id: [{uid}]");
    assert_eq!(uid, "cm-f924e4ca-cf67-6835-e74c-e0a569319372!m");

    // clean up db
    let result = fs::remove_file("00000000-0000-0000-0000-222222222222.world");
    log::info!("File Cleanup Result: {result:?}");
}
#[test]
fn test_can_get_from_non_zero_layer() {
    setup_test_logger();
    let guid = Some(Uuid::parse_str("00000000-0000-0000-0000-111111111111").unwrap());
    let cm = make_test_chunk_manager(8, 8, 4, 1024, 8, 8, 1, true, guid);

    log::info!("[INITIAL]");
    cm.log_all_layer_index_diagnostics(false);

    let c = 2_isize.pow(4);
    log::info!("Looking at ({c}, {c}, 3)");
    let test_val = cm.get_at(c, c, 3).unwrap();
    log::info!("test_val: {test_val:02X}");

    log::info!("[INTERIM]");
    cm.log_all_layer_index_diagnostics(false);
    for l in 0..5 {
        let bounds = cm.get_bounds_for_layer(l, true);
        log::info!("Layer {l} bounds: {bounds:?}");
    }

    let padded_bounds_3 = cm
        .get_bounds_for_layer(3, true)
        .expect("Layer bounds error");

    let cropped_bounds_3 = cm
        .get_bounds_for_layer(3, false)
        .expect("Layer bounds error");

    let (padded_x_min, padded_y_min, padded_x_max, padded_y_max) = padded_bounds_3;
    let (_cropped_x_min, _cropped_y_min, _cropped_x_max, _cropped_y_max) = cropped_bounds_3;
    log::info!("Layer 3 padded bounds: {padded_bounds_3:?}");
    log::info!("Layer 3 cropped bounds: {padded_bounds_3:?}");
    log::info!(
        "Prior bounds: (0, 0, {}, {})",
        cm.width as isize * 8,
        cm.height as isize * 8
    );
    // return;
    for y in padded_y_min..padded_y_max {
        for x in padded_x_min..padded_x_max {
            let v3 = cm.get_at(x, y, 3).unwrap();
            let (x0, y0) = (x / 8, y / 8);
            let v0 = cm.get_at(x0, y0, 0).unwrap();
            let v3s = std::format!("{v3:02X}");
            let v0s = std::format!("{v0:02X}");
            // if x >= cropped_x_min && y >= cropped_y_min && x < cropped_x_max && y < cropped_y_max {
            //     assert_eq!(v3s, v0s, "({x} {y}, 3):[{v3s}] -> ({x0}, {y0}, 0):[{v0s}]");
            // }
        }
    }
    log::info!("[FINAL]");
    cm.log_all_layer_index_diagnostics(true);

    cm.log_all_layer_value_diagnostics(true);

    // ChunkManager: [manager_da1ea0e2-cd27-6636-0cc8-d8ce3907b8a5!m]
    // Layer: [0]:[layer_7ff3bba6-6731-e753-7a27-fd10caddb999!m] - VALUES
    // Layer: [1]:[layer_33fc7b80-e69f-0b90-18ca-dd7d09624192!m] - VALUES
    // Layer: [2]:[layer_2a6ad74a-654c-a84d-3afd-0f03f53dc2df!m] - VALUES
    // Layer: [3]:[layer_184ce058-9441-5253-387c-2f7c95821aff!m] - VALUES
}
