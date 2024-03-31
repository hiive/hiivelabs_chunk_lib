use crate::bounds::Bounds;
use crate::chunk::Chunk;
use crate::chunk_layer::TIndex;
use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Instant;

#[test]
fn chunk_creation() {
    let chunk = Chunk::new(0, 0, 10, 10, 1);
    assert_eq!(chunk.bounds.width, 10);
    assert_eq!(chunk.bounds.height, 10);
    assert_eq!(chunk.bounds.padding, 1);
    // Ensure the vector is correctly sized with padding
    assert_eq!(chunk.tiles.len(), 144); // (10 + 2*1) * (10 + 2*1)
}

#[test]
fn get_at_for_empty_tile() {
    let chunk = Chunk::new(0, 0, 10, 10, 1);
    assert!(chunk.get_at(5, 5).is_none());
}

#[test]
fn set_at_for_occupied_tile() {
    let mut chunk = Chunk::new(0, 0, 10, 10, 1);
    let _ = chunk.set_at(5, 5, 42); // Assuming ChunkTile takes an i32 for this example
    assert_eq!(chunk.get_at(5, 5).unwrap(), 42);
    let _ = chunk.set_at(5, 5, 43); // Assuming ChunkTile takes an i32 for this example
    assert_eq!(chunk.get_at(5, 5).unwrap(), 43);
}

#[test]
fn set_and_get_tile() {
    let mut chunk = Chunk::new(0, 0, 10, 10, 1);
    let _ = chunk.set_at(5, 5, 42); // Assuming ChunkTile takes an i32 for this example
    assert_eq!(chunk.get_at(5, 5).unwrap(), 42);
}

#[test]
fn set_at_out_of_bounds() {
    let mut chunk = Chunk::new(0, 0, 10, 10, 1);
    let result = chunk.set_at(50, 50, 42);
    assert!(result.is_err());
}

#[test]
fn is_not_complete() {
    let chunk = Chunk::new(0, 0, 10, 10, 1);
    assert!(!chunk.is_complete())
}

fn generate_random_vector(length: usize) -> Vec<usize> {
    // let seed = [42; 32];
    // let mut rng = StdRng::from_seed(seed);
    let mut rng = StdRng::from_rng(rand::thread_rng()).unwrap();
    (0..length).map(|_| rng.gen()).collect()
}

fn build_complete_chunk(width: usize, height: usize, padding: usize, is_random: bool) -> Chunk {
    let mut chunk = Chunk::new(0, 0, width, height, padding);

    let x_min = -(chunk.bounds.padding as isize);
    let x_max = (chunk.bounds.width + chunk.bounds.padding) as isize;
    let y_min = -(chunk.bounds.padding as isize);
    let y_max = (chunk.bounds.height + chunk.bounds.padding) as isize;
    let size = chunk.chunk_width * chunk.chunk_height;
    let values_to_use: Vec<usize> = {
        if is_random {
            generate_random_vector(size)
        } else {
            (0_usize..(chunk.chunk_width * chunk.chunk_height)).collect()
        }
    };
    let mut ix = 0;
    for y in y_min..y_max {
        for x in x_min..x_max {
            // println!("({x}, {y})");
            let _ = chunk.set_at(x, y, values_to_use[ix]);
            ix += 1;
        }
    }
    chunk
}

#[test]
fn encode_decode_test() {
    let chunk = build_complete_chunk(64, 32, 1, true);
    let chunk_mem_size = std::mem::size_of::<Bounds>()
        + 2 * std::mem::size_of::<usize>()
        + chunk.tiles.len() * std::mem::size_of::<TIndex>();
    println!("chunk length: {chunk_mem_size}");

    let start = Instant::now(); // Start timing
                                // let first_start = start;
                                // serialize
    let encoded: Vec<u8> = bitcode::encode(&chunk);
    let duration = start.elapsed(); // End timing
    println!("encoded length: {}, time: {duration:?}", encoded.len());

    // deserialize
    let start = Instant::now(); // Start timing
    let decoded: Chunk = bitcode::decode(&encoded).unwrap();
    let duration = start.elapsed(); // End timing
    println!("decode time: {duration:?}");

    assert_eq!(chunk, decoded);

    // compress
    let start = Instant::now(); // Start timing
    let compressed = compress_to_vec(encoded.as_slice(), 6);
    let duration = start.elapsed(); // End timing
    println!(
        "compressed length: {}, time: {duration:?}",
        compressed.len()
    );

    let pct_reduction = (1000.0 * compressed.len() as f32 / chunk_mem_size as f32).round() / 10.;
    println!("%ge of original size: {pct_reduction}");

    // decompress
    let start = Instant::now(); // Start timing
    let decompressed = decompress_to_vec(compressed.as_slice()).unwrap();
    let duration = start.elapsed(); // End timing
    println!("decompress time: {duration:?}");
    assert_eq!(encoded, decompressed);
    assert_eq!(encoded.len(), decompressed.len());

    // deserialize decompressed
    let decoded: Chunk = bitcode::decode(&decompressed).unwrap();
    assert_eq!(chunk, decoded);
}

#[test]
fn is_complete() {
    let chunk = build_complete_chunk(10, 10, 1, true);
    assert!(chunk.is_complete())
}

#[test]
fn set_get_set_get_test() {
    let mut chunk: Chunk = Chunk::new(10, 10, 20, 10, 1);

    // chunk.init_chunk_tile(9, 9, 99);
    let _ = chunk.set_at(9, 9, 101);

    {
        let test = chunk.get_at(9, 9);
        // let tv = test.unwrap_or(&u32::MAX);

        println!("{test:?}");
    }

    let _ = chunk.set_at(9, 9, 102);
    let test = chunk.get_at(9, 9);

    println!("{test:?}");
    //let tv = test.unwrap_or(&u32::MAX);
}
