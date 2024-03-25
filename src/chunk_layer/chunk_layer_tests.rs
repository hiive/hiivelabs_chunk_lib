#[cfg(test)]
use super::ChunkLayer;

#[test]
fn test_is_chunk_border_coord() {
    let cm = crate::chunk_manager::chunk_manager_tests::make_test_chunk_manager(
        64, 32, 4, 32, 16, 1, false,
    );
    let layer = ChunkLayer::<u8>::new(1, 10, 10, 1, 10, 10);
    assert_eq!(layer.is_chunk_border_coord(10, 10), (true, true));
    assert_eq!(layer.is_chunk_border_coord(5, 5), (false, false));
}

#[test]
fn test_tile_coords_to_chunk_coords() {
    let cm = crate::chunk_manager::chunk_manager_tests::make_test_chunk_manager(
        64, 32, 4, 32, 16, 1, false,
    );
    let layer = ChunkLayer::<u8>::new(1, 10, 10, 1, 10, 10);
    assert_eq!(layer.tile_coords_to_chunk_coords(15, 25), (1, 2));
}

#[test]
fn test_chunk_coords_to_tile_coords() {
    let cm = crate::chunk_manager::chunk_manager_tests::make_test_chunk_manager(
        64, 32, 4, 32, 16, 1, false,
    );
    let layer = ChunkLayer::<u8>::new(1, 10, 10, 1, 10, 10);
    assert_eq!(layer.chunk_coords_to_tile_coords(1, 2), (10, 20));
}

#[test]
fn test_get_chunk_indices_for_tile_coords() {
    let cm = crate::chunk_manager::chunk_manager_tests::make_test_chunk_manager(
        64, 32, 4, 32, 16, 1, false,
    );
    let layer = ChunkLayer::<u8>::new(1, 2, 2, 1, 10, 10);
    // Assuming Bounds::get_index_for_coords and Bounds::is_in_bounds are correctly implemented
    // and chunks are properly initialized in the layer.
    // This example assumes chunks are laid out linearly and checks for boundary conditions.
    // Adjust the logic based on how your chunks are indexed and stored.

    // at the top-left boundary
    let indices = layer.get_chunk_indices_for_tile_coords(0, 0);
    assert_eq!(indices.len(), 1);
    assert!(indices.contains(&0));

    // at the bottom right boundary
    let indices = layer.get_chunk_indices_for_tile_coords(20, 20);
    assert_eq!(indices.len(), 1);
    assert!(indices.contains(&3));

    // at the top right boundary
    let indices = layer.get_chunk_indices_for_tile_coords(20, 0);
    assert_eq!(indices.len(), 1);
    println!("{indices:?}");
    assert!(indices.contains(&1));

    // at the bottom left boundary
    let indices = layer.get_chunk_indices_for_tile_coords(0, 20);
    assert_eq!(indices.len(), 1);
    println!("{indices:?}");
    assert!(indices.contains(&2));

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
    assert!(indices.contains(&3));

    // On a chunk boundary
    let indices = layer.get_chunk_indices_for_tile_coords(10, 10);
    // assert_eq!(main_chunk, main_chunk_2);
    assert_eq!(indices.len(), 4); // Expect multiple indices due to boundary condition
    println!("{indices:?}");
    assert!(indices.contains(&0));
    assert!(indices.contains(&1));
    assert!(indices.contains(&2));
    assert!(indices.contains(&3));

    // on a left edge
    let indices = layer.get_chunk_indices_for_tile_coords(10, 0);
    assert_eq!(indices.len(), 2);
    println!("{indices:?}");
    assert!(indices.contains(&0));
    assert!(indices.contains(&1));
}

// Add more tests as needed for other methods and edge cases.
