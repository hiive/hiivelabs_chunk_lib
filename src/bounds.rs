use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub(crate) struct Bounds {
    pub(crate) x: isize,
    pub(crate) y: isize,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) padding: usize,
    pub(crate) assert_on_out_of_bounds: bool,
}
impl Bounds {
    pub(crate) fn get_index_for_coords(&self, x: isize, y: isize) -> (isize, isize, isize) {
        let padding = self.padding as isize;
        let width = self.width as isize;
        let height = self.height as isize;
        let (adj_x, adj_y) = (x + padding - self.x, y + padding - self.y);

        // #[cfg(debug_assertions)]
        // {
        //     let sx = self.x;
        //     let sy = self.y;
        //     println!("Bounds::get_index_for_coords: self:({sx}, {sy}) pad:({padding})");
        //     println!("Bounds::get_index_for_coords: ({x}, {y}) -> adj:({adj_x}, {adj_y})");
        // }

        let padded_width = width + 2 * padding;
        let padded_height = height + 2 * padding;
        if self.assert_on_out_of_bounds {
            assert!(
                (adj_x >= 0) && (adj_x < padded_width) && (adj_y >= 0) && (adj_y < padded_height),
                "Chunk coordinates are out of bounds"
            );
        }

        let ix = (adj_y * padded_width + adj_x, adj_x, adj_y);

        ix
    }

    /*
    pub(crate) fn get_bounds_min_max(&self) -> (isize, isize) {
        let padding = self.padding as isize;
        let width = self.width as isize;
        let height = self.width as isize;
        let x_min = -padding;
        let y_min = -padding;
        let x_max = width + padding;
        let y_max = height + padding;
        let x_size = x_max - x_min;
        let y_size = y_max - y_min;
        let min = x_min + (y_min * x_size);
        let max = (y_size * x_size) - min;
        (min, max)
    }
     */

    pub(crate) fn is_in_bounds(&self, ix: isize) -> bool {
        let max_ix = (self.width + 2 * self.padding) * (self.height + 2 * self.padding);
        ix >= 0 && ix < max_ix as isize
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
// tests
////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
mod bounds_tests;
