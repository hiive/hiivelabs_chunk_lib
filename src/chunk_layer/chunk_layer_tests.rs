use crate::chunk_layer::{ChunkLayer, TIndex};
use crate::test_utils::setup_test_logger;
use uuid::Uuid;

#[test]
fn test_is_chunk_border_coord() {
    setup_test_logger();
    let oob: Option<TIndex> = Some(0);
    let guid_bytes = Uuid::new_v4().as_bytes().to_owned();
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let layer = ChunkLayer::new(prev_layer, 1, guid_bytes, 32, 10, 10, 1, 10, 10, oob);
    assert_eq!(layer.is_chunk_border_coord(10, 10), (true, true));
    assert_eq!(layer.is_chunk_border_coord(5, 5), (false, false));
}

#[test]
fn test_tile_coords_to_chunk_coords() {
    setup_test_logger();
    let oob: Option<TIndex> = Some(0);
    let guid_bytes = Uuid::new_v4().as_bytes().to_owned();
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let layer = ChunkLayer::new(prev_layer, 1, guid_bytes, 32, 10, 10, 1, 10, 10, oob);
    assert_eq!(layer.tile_coords_to_chunk_coords(15, 25), (1, 2));
}

#[test]
fn test_chunk_coords_to_tile_coords() {
    setup_test_logger();
    let oob: Option<TIndex> = Some(0);
    let guid_bytes = Uuid::new_v4().as_bytes().to_owned();
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let layer = ChunkLayer::new(prev_layer, 1, guid_bytes, 32, 10, 10, 1, 10, 10, oob);
    assert_eq!(layer.chunk_coords_to_tile_coords(1, 2), (10, 20));
}

#[test]
fn test_get_chunk_indices_for_tile_coords() {
    setup_test_logger();
    let oob: Option<TIndex> = Some(0);
    let guid_bytes = Uuid::new_v4().as_bytes().to_owned();
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let layer = ChunkLayer::new(prev_layer, 1, guid_bytes, 32, 2, 2, 1, 10, 10, oob);
    // Assuming Bounds::get_index_for_coords and Bounds::is_index_in_bounds are correctly implemented
    // and chunks are properly initialized in the layer.
    // This example assumes chunks are laid out linearly and checks for boundary conditions.
    // Adjust the logic based on how your chunks are indexed and stored.

    assert_eq!(layer.tile_bounds.width, 20);
    assert_eq!(layer.tile_bounds.height, 20);
    assert_eq!(layer.tile_bounds.padding, 1);
    let padded_bounds = layer.tile_bounds.get_bound_coords(true);
    let non_padded_bounds = layer.tile_bounds.get_bound_coords(false);
    assert_eq!(padded_bounds, (-1, -1, 21, 21));
    assert_eq!(non_padded_bounds, (0, 0, 20, 20));
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
    log::info!("{indices:?}");
    assert!(indices.contains(&(1, (1, 0))));

    // at the bottom left boundary
    let indices = layer.get_chunk_indices_for_tile_coords(0, 20);
    assert_eq!(indices.len(), 1);
    log::info!("{indices:?}");
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
    log::info!("{indices:?}");
    assert!(indices.contains(&(3, (1, 1))));

    // On a chunk boundary
    let indices = layer.get_chunk_indices_for_tile_coords(10, 10);
    // assert_eq!(main_chunk, main_chunk_2);
    assert_eq!(indices.len(), 4); // Expect multiple indices due to boundary condition
    log::info!("{indices:?}");
    assert!(indices.contains(&(0, (0, 0))));
    assert!(indices.contains(&(1, (1, 0))));
    assert!(indices.contains(&(2, (0, 1))));
    assert!(indices.contains(&(3, (1, 1))));

    // on a left edge
    let indices = layer.get_chunk_indices_for_tile_coords(10, 0);
    assert_eq!(indices.len(), 2);
    log::info!("{indices:?}");
    assert!(indices.contains(&(0, (0, 0))));
    assert!(indices.contains(&(1, (1, 0))));
}

#[test]
fn test_set_and_get_top_layer_failing_case() {
    setup_test_logger();
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let oob: Option<TIndex> = Some(0);
    let guid_bytes = Uuid::new_v4().as_bytes().to_owned();
    let mut layer = ChunkLayer::new(prev_layer, 0, guid_bytes, 32, 1, 1, 1, 10, 10, oob);

    //for i in -1_isize..5 {
    let x = 2;
    let y = 7;
    let value: TIndex = (x + y) as TIndex;

    log::trace!("{x}, {y}");

    let _ = layer.set_at(x, y, value);
    let r_value = layer.get_at(x, y).expect("Value should be set");
    assert_eq!(r_value, value)
    //}
}

#[test]
fn test_set_and_get_top_layer() {
    setup_test_logger();
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let oob: Option<TIndex> = Some(0);
    let guid_bytes = Uuid::new_v4().as_bytes().to_owned();
    let mut layer = ChunkLayer::new(prev_layer, 0, guid_bytes, 32, 1, 1, 1, 10, 10, oob);

    for i in -1_isize..5 {
        let x = i;
        let y = layer.tile_bounds.height as isize - (i + 1);
        let value: TIndex = (x + y) as TIndex;

        log::trace!("{x}, {y}");

        let _ = layer.set_at(x, y, value);
        let r_value = layer.get_at(x, y).expect("Value should be set");
        if layer.tile_bounds.is_coords_in_bounds(x, y, false) {
            assert_eq!(r_value, value)
        } else {
            assert_eq!(r_value, oob.unwrap())
        }
    }
}

#[test]
fn test_boundary_chunk_values_set() {
    setup_test_logger();
    let prev_layer = ChunkLayer::make_layer_rc(None);
    let oob: Option<TIndex> = Some(0);
    let guid_bytes = Uuid::new_v4().as_bytes().to_owned();
    let mut layer = ChunkLayer::new(prev_layer, 0, guid_bytes, 32, 2, 2, 1, 4, 4, oob);

    let x = 4;
    let y = 4;
    // set at the point where 4 chunks overlap
    let _ = layer.set_at(x, y, 2);
    let indices = layer.get_chunk_indices_for_tile_coords(x, y);
    // assert_eq!(main_chunk, main_chunk_2);
    // assert_eq!(indices.len(), 4); // Expect multiple indices due to boundary condition
    log::trace!("{indices:?}");

    log::info!("Query coords: ({x}, {y})");
    for (ix, _) in &indices {
        let mut chunks_ref = layer.chunks.borrow_mut();
        let chunk = chunks_ref.get(ix).unwrap();
        let usc = chunk.get_unset_tile_count();
        let wh = chunk.bounds.get_tile_count(true);
        let vf = chunk.get_at(x, y).expect("should be set!");
        log::trace!("{usc}/{wh} : {vf}");
        assert_eq!(usc, wh - 1);
        assert_eq!(vf, 2);
        let (ox, oy) = (chunk.bounds.x, chunk.bounds.y);
        let (xx, yy) = chunk.bounds.get_adjusted_coordinates(x, y);
        log::info!("Chunk [{ix}] Origin: ({ox}, {oy}),  Adjusted (x, y): ({xx}, {yy})");
        assert_eq!(chunk.get_at(x - 1, y), None);
    }
}

// #[test]
// fn test_set_and_get_layer1() {
//     let mut layer0 = ChunkLayer::new(ChunkLayer::make_layer_rc(None),
//                                      0,
//                                      32,
//                                      1,
//                                      1,
//                                      1,
//                                      16,
//                                      8);
//
//     // fill up layer 0
//     let w0 = layer0.width_in_tiles as isize;
//     let h0 = layer0.height_in_tiles as isize;
//
//     for y in 0..h0 {
//         for x in 0..w0 {
//             let v:TIndex = (y * w0 + x) as TIndex;
//             layer0.set_at(x, y, v);
//         }
//     }
//
//     let mut layer0= ChunkLayer::make_layer_rc(Some(layer0));
//     let mut layer1 = ChunkLayer::new(layer0.clone(),
//                                      1,
//                                      32,
//                                      2,
//                                      2,
//                                      1,
//                                      16,
//                                      8);
//
//     // now let's query the layer 1 values.
//     let w1 = layer1.width_in_tiles as isize;
//     let h1 = layer1.height_in_tiles as isize;
//
//     // let mut layer0_opt = layer0;
//     let mut layer0_borrowed =  layer0.borrow_mut();
//     if let Some(layer0) = layer0_borrowed.as_mut()
//     {
//         for y in 0..h1 {
//             for x in 0..w1 {
//                 let v0 = layer0.get_at(x / 2, y / 2);
//                 let v1 = layer1.get_at(x, y);
//                 assert_eq!(v0, v1);
//             }
//         }
//     }
//
//
// }
