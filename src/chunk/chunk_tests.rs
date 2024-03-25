#[cfg(test)]
use crate::chunk::Chunk;

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
    chunk.set_at(5, 5, 42); // Assuming ChunkTile takes an i32 for this example
    assert_eq!(chunk.get_at(5, 5).unwrap(), 42);
    chunk.set_at(5, 5, 43); // Assuming ChunkTile takes an i32 for this example
    assert_eq!(chunk.get_at(5, 5).unwrap(), 43);
}

#[test]
fn set_and_get_tile() {
    let mut chunk = Chunk::new(0, 0, 10, 10, 1);
    chunk.set_at(5, 5, 42); // Assuming ChunkTile takes an i32 for this example
    assert_eq!(chunk.get_at(5, 5).unwrap(), 42);
}

#[test]
#[should_panic(expected = "Chunk coordinates are out of bounds")]
fn set_at_out_of_bounds() {
    let mut chunk = Chunk::new(0, 0, 10, 10, 1);
    chunk.set_at(50, 50, 42); // This should panic
}

#[test]
fn is_not_complete() {
    let chunk = Chunk::new(0, 0, 10, 10, 1);
    assert!(!chunk.is_complete())
}

#[test]
fn is_complete() {
    let mut chunk = Chunk::new(0, 0, 10, 10, 1);

    let x_min = -(chunk.bounds.padding as isize);
    let x_max = (chunk.bounds.width + chunk.bounds.padding) as isize;
    let y_min = -(chunk.bounds.padding as isize);
    let y_max = (chunk.bounds.height + chunk.bounds.padding) as isize;

    let values_to_use: Vec<usize> =
        (0_usize..(chunk.chunk_width * chunk.chunk_height) as usize).collect();
    let mut ix = 0;
    for y in y_min..y_max {
        for x in x_min..x_max {
            println!("({x}, {y})");
            chunk.set_at(x, y, values_to_use[ix]);
            ix += 1;
        }
    }

    assert!(chunk.is_complete())
}

#[test]
fn set_get_set_get_test() {
    let mut chunk: Chunk = Chunk::new(10, 10, 20, 10, 1);

    // chunk.init_chunk_tile(9, 9, 99);
    chunk.set_at(9, 9, 101);

    {
        let test = chunk.get_at(9, 9);
        // let tv = test.unwrap_or(&u32::MAX);

        println!("{test:?}");
    }

    chunk.set_at(9, 9, 102);
    let test = chunk.get_at(9, 9);

    println!("{test:?}");
    //let tv = test.unwrap_or(&u32::MAX);
}
