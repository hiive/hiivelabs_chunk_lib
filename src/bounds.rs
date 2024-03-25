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
        let (x, y) = (x + padding - self.x, y + padding - self.y);
        if self.assert_on_out_of_bounds {
            assert!(
                (x >= -padding)
                    && (x <= width + padding)
                    && (y >= -padding)
                    && (y <= height + padding),
                "Chunk coordinates are out of bounds"
            );
        }

        (y * (width + 2 * padding) + x, x, y)
    }

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
