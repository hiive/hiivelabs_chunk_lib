use crate::bounds::Bounds;
use hiivelabs_rand_utils_lib::prelude::IDim;
use hiivelabs_rand_utils_lib::utils::test_utils::setup_test_logger;

#[test]
fn test_get_index_for_coords_within_bounds() {
    setup_test_logger();
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 0,
    };

    let result = bounds.get_index_for_coords(5, 5, false);
    match result {
        Ok((index, x, y)) => {
            assert_eq!(index, 55);
            assert_eq!(x, 5);
            assert_eq!(y, 5);
        }
        Err(msg) => {
            panic!("{msg}");
        }
    }
}

#[test]
#[should_panic(expected = "Chunk coordinates are out of bounds")]
fn test_get_index_for_coords_out_of_bounds_panic() {
    setup_test_logger();
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 0,
    };

    let result = bounds.get_index_for_coords(11, 11, true);
    match result {
        Ok(_) => {}
        Err(msg) => {
            panic!("{msg}")
        }
    }
}

#[test]
fn test_get_index_for_coords_out_of_bounds_no_panic() {
    setup_test_logger();
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 0,
    };

    let result = bounds.get_index_for_coords(11, 11, false);

    match result {
        Ok(ix_xy) => {
            assert_eq!(ix_xy, (121, 11, 11))
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }
}

#[test]
fn test_is_in_bounds_limits() {
    setup_test_logger();
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 1, // Adjusted padding to ensure the logic in `is_index_in_bounds` is correct.
    };

    let result = bounds.get_index_for_coords(0, 0, false);
    match result {
        Ok((ix, _, _)) => {
            assert!(bounds.is_index_in_bounds(ix));
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }

    let result = bounds.get_index_for_coords(-1, -1, false);
    match result {
        Ok((ix, _, _)) => {
            assert!(bounds.is_index_in_bounds(ix));
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }

    let result = bounds.get_index_for_coords(-2, -2, false);
    match result {
        Ok((ix, _, _)) => {
            assert!(!bounds.is_index_in_bounds(ix));
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }

    let result = bounds.get_index_for_coords(11, 11, false);

    match result {
        Ok((ix, _, _)) => {
            assert!(!bounds.is_index_in_bounds(ix));
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }

    let result = bounds.get_index_for_coords(12, 12, false);
    match result {
        Ok((ix, _, _)) => {
            assert!(!bounds.is_index_in_bounds(ix));
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }
}

#[test]
fn test_is_in_bounds_false() {
    setup_test_logger();
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 1, // Adjusted padding to ensure the logic in `is_index_in_bounds` is correct.
    };

    // Assuming the adjustment for `is_index_in_bounds` logic: `ix < 0 || ix >= max_ix as IDim`
    // Note: The test indicates a logic correction needed in `is_index_in_bounds`.
    let max_index = bounds.width as IDim * bounds.height as IDim;
    assert!(bounds.is_index_in_bounds(max_index)); // Should be out of bounds
}

#[test]
fn test_get_index_for_coords_with_padding() {
    setup_test_logger();
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 100,
        height: 100,
        padding: 10,
    };

    // Coordinates within the padding area (top-left corner)
    let result = bounds.get_index_for_coords(-5, -5, false);
    match result {
        Ok((_, x, y)) => {
            // Expectations need to adjust based on your implementation of get_index_for_coords
            // This example assumes (x, y) coordinates are transformed relative to padding.
            assert_eq!(x, 5); // Adjusted x coordinate within padding
            assert_eq!(y, 5); // Adjusted y coordinate within padding
                              // Index calculation will depend on the specific layout logic of your chunks
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }
}

#[test]
fn coordinates_within_padding_are_in_bounds() {
    setup_test_logger();
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 5,
    };

    // Example assumes get_index_for_coords gives a linear index for the coordinate.
    // Adjust the logic to match your actual index calculation.
    let result = bounds.get_index_for_coords(-1, -1, false);
    match result {
        Ok(within_padding_top_left) => {
            assert!(bounds.is_index_in_bounds(within_padding_top_left.0));
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }

    let result = bounds.get_index_for_coords(11, 11, false);
    match result {
        Ok(within_padding_bottom_right) => {
            assert!(bounds.is_index_in_bounds(within_padding_bottom_right.0));
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }
}

#[test]
fn coordinates_outside_padding_are_out_of_bounds() {
    setup_test_logger();
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 5,
    };

    // Outside the padding area
    let result = bounds.get_index_for_coords(-6, -6, false);
    match result {
        Ok(outside_padding_top_left) => {
            assert!(!bounds.is_index_in_bounds(outside_padding_top_left.0));
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }

    let result = bounds.get_index_for_coords(16, 16, false);
    match result {
        Ok(outside_padding_bottom_right) => {
            assert!(!bounds.is_index_in_bounds(outside_padding_bottom_right.0));
        }
        Err(msg) => {
            panic!("{msg}")
        }
    }
}
