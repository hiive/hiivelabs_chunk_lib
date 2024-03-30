use crate::chunk_layer::{ChunkLayer, TIndex};

#[test]
fn test_is_chunk_border_coord() {
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let layer = ChunkLayer::new(prev_layer, 1, 32, 10, 10, 1, 10, 10);
    assert_eq!(layer.is_chunk_border_coord(10, 10), (true, true));
    assert_eq!(layer.is_chunk_border_coord(5, 5), (false, false));
}

#[test]
fn test_tile_coords_to_chunk_coords() {
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let layer = ChunkLayer::new(prev_layer, 1, 32, 10, 10, 1, 10, 10);
    assert_eq!(layer.tile_coords_to_chunk_coords(15, 25), (1, 2));
}

#[test]
fn test_chunk_coords_to_tile_coords() {
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let layer = ChunkLayer::new(prev_layer, 1, 32, 10, 10, 1, 10, 10);
    assert_eq!(layer.chunk_coords_to_tile_coords(1, 2), (10, 20));
}

#[test]
fn test_get_chunk_indices_for_tile_coords() {
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let layer = ChunkLayer::new(prev_layer, 1, 32, 2, 2, 1, 10, 10);
    // Assuming Bounds::get_index_for_coords and Bounds::is_in_bounds are correctly implemented
    // and chunks are properly initialized in the layer.
    // This example assumes chunks are laid out linearly and checks for boundary conditions.
    // Adjust the logic based on how your chunks are indexed and stored.

    // at the top-left boundary
    let indices = layer.get_chunk_indices_for_tile_coords(0, 0);
    assert_eq!(indices.len(), 1);
    assert!(indices.contains(&(0, (0, 0))));

    // at the bottom right boundary
    let indices = layer.get_chunk_indices_for_tile_coords(20, 20);
    assert_eq!(indices.len(), 1);
    assert!(indices.contains(&(3, (1, 1))));

    // at the top right boundary
    let indices = layer.get_chunk_indices_for_tile_coords(20, 0);
    assert_eq!(indices.len(), 1);
    println!("{indices:?}");
    assert!(indices.contains(&(1, (1, 0))));

    // at the bottom left boundary
    let indices = layer.get_chunk_indices_for_tile_coords(0, 20);
    assert_eq!(indices.len(), 1);
    println!("{indices:?}");
    assert!(indices.contains(&(2, (0, 1))));

    // out of bounds (-ve)
    let indices = layer.get_chunk_indices_for_tile_coords(-5, -5);
    assert_eq!(indices.len(), 0);

    // out of bounds (+ve)
    let indices = layer.get_chunk_indices_for_tile_coords(25, 25);
    assert_eq!(indices.len(), 0);

    // Directly within a chunk
    let indices = layer.get_chunk_indices_for_tile_coords(11, 11);
    assert_eq!(indices.len(), 1);
    println!("{indices:?}");
    assert!(indices.contains(&(3, (1, 1))));

    // On a chunk boundary
    let indices = layer.get_chunk_indices_for_tile_coords(10, 10);
    // assert_eq!(main_chunk, main_chunk_2);
    assert_eq!(indices.len(), 4); // Expect multiple indices due to boundary condition
    println!("{indices:?}");
    assert!(indices.contains(&(0, (0, 0))));
    assert!(indices.contains(&(1, (1, 0))));
    assert!(indices.contains(&(2, (0, 1))));
    assert!(indices.contains(&(3, (1, 1))));

    // on a left edge
    let indices = layer.get_chunk_indices_for_tile_coords(10, 0);
    assert_eq!(indices.len(), 2);
    println!("{indices:?}");
    assert!(indices.contains(&(0, (0, 0))));
    assert!(indices.contains(&(1, (1, 0))));
}

#[test]
fn test_set_and_get_top_layer_failing_case() {
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let mut layer = ChunkLayer::new(prev_layer, 1, 32, 1, 1, 1, 10, 10);

    //for i in -1_isize..5 {
    let x = 2;
    let y = 7;
    let value: TIndex = (x + y) as TIndex;

    println!("{x}, {y}");

    layer.set_at(x, y, value);
    let r_value = layer.get_at(x, y).expect("Value should be set");
    assert_eq!(r_value, value)
    //}
}

#[test]
fn test_set_and_get_top_layer() {
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let mut layer = ChunkLayer::new(prev_layer, 1, 32, 1, 1, 1, 10, 10);

    for i in -1_isize..5 {
        let x = i;
        let y = layer.height_in_tiles as isize - (i + 1);
        let value: TIndex = (x + y) as TIndex;

        println!("{x}, {y}");

        layer.set_at(x, y, value);
        let r_value = layer.get_at(x, y).expect("Value should be set");
        assert_eq!(r_value, value)
    }
}
