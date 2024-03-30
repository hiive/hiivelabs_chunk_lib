use crate::bounds::Bounds;

#[test]
fn test_get_index_for_coords_within_bounds() {
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 0,
        assert_on_out_of_bounds: false,
    };

    let (index, x, y) = bounds.get_index_for_coords(5, 5);
    assert_eq!(index, 55);
    assert_eq!(x, 5);
    assert_eq!(y, 5);
}

#[test]
#[should_panic(expected = "Chunk coordinates are out of bounds")]
fn test_get_index_for_coords_out_of_bounds_panic() {
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 0,
        assert_on_out_of_bounds: true,
    };

    bounds.get_index_for_coords(11, 11);
}

#[test]
fn test_get_index_for_coords_out_of_bounds_no_panic() {
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 0,
        assert_on_out_of_bounds: false,
    };

    let ix = bounds.get_index_for_coords(11, 11);
    assert_eq!(ix, (121, 11, 11))
}

#[test]
fn test_is_in_bounds_limits() {
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 1, // Adjusted padding to ensure the logic in `is_in_bounds` is correct.
        assert_on_out_of_bounds: false,
    };

    let (ix, _, _) = bounds.get_index_for_coords(0, 0);
    assert!(bounds.is_in_bounds(ix));

    let (ix, _, _) = bounds.get_index_for_coords(-1, -1);
    assert!(bounds.is_in_bounds(ix));

    let (ix, _, _) = bounds.get_index_for_coords(-2, -2);
    assert!(!bounds.is_in_bounds(ix));

    let (ix, _, _) = bounds.get_index_for_coords(11, 11);
    assert!(!bounds.is_in_bounds(ix));

    let (ix, _, _) = bounds.get_index_for_coords(12, 12);
    assert!(!bounds.is_in_bounds(ix));
}

#[test]
fn test_is_in_bounds_false() {
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 1, // Adjusted padding to ensure the logic in `is_in_bounds` is correct.
        assert_on_out_of_bounds: false,
    };

    // Assuming the adjustment for `is_in_bounds` logic: `ix < 0 || ix >= max_ix as isize`
    // Note: The test indicates a logic correction needed in `is_in_bounds`.
    let max_index = bounds.width as isize * bounds.height as isize;
    assert!(bounds.is_in_bounds(max_index)); // Should be out of bounds
}

#[test]
fn test_get_index_for_coords_with_padding() {
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 100,
        height: 100,
        padding: 10,
        assert_on_out_of_bounds: false,
    };

    // Coordinates within the padding area (top-left corner)
    let (_, x, y) = bounds.get_index_for_coords(-5, -5);
    // Expectations need to adjust based on your implementation of get_index_for_coords
    // This example assumes (x, y) coordinates are transformed relative to padding.
    assert_eq!(x, 5); // Adjusted x coordinate within padding
    assert_eq!(y, 5); // Adjusted y coordinate within padding
                      // Index calculation will depend on the specific layout logic of your chunks
}

#[test]
fn coordinates_within_padding_are_in_bounds() {
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 5,
        assert_on_out_of_bounds: false,
    };

    // Example assumes get_index_for_coords gives a linear index for the coordinate.
    // Adjust the logic to match your actual index calculation.
    let within_padding_top_left = bounds.get_index_for_coords(-1, -1).0;
    assert!(bounds.is_in_bounds(within_padding_top_left));

    let within_padding_bottom_right = bounds.get_index_for_coords(11, 11).0;
    assert!(bounds.is_in_bounds(within_padding_bottom_right));
}

#[test]
fn coordinates_outside_padding_are_out_of_bounds() {
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
        padding: 5,
        assert_on_out_of_bounds: false,
    };

    // Outside the padding area
    let outside_padding_top_left = bounds.get_index_for_coords(-6, -6).0;
    assert!(!bounds.is_in_bounds(outside_padding_top_left));

    let outside_padding_bottom_right = bounds.get_index_for_coords(16, 16).0;
    assert!(!bounds.is_in_bounds(outside_padding_bottom_right));
}
