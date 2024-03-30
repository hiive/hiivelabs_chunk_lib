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
    /// Retrieve the index for the specified coordinates, assuming that the values
    /// are stored in a Vec row-first. The calculation includes the padding variable.
    /// It returns a tuple (index, adjusted_x, adjusted_y)
    /// where index is the index into the Vec, and adjusted_x, adjusted_y are the
    /// tile coordinates w.r.t. the top-left corner of this chunk.
    /// (0, 0) represents the top-left tile that is not in the padding area.
    /// (-1, -1) would indicate the tile to the top-left of that (assuming that the padding
    /// was at least 1.)
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
        (adj_y * padded_width + adj_x, adj_x, adj_y)
    }

    /// This confirms that the specified index is in bounds.
    pub(crate) fn is_in_bounds(&self, ix: isize) -> bool {
        let max_ix = (self.width + (2 * self.padding)) * (self.height + (2 * self.padding));
        ix >= 0 && ix < max_ix as isize
    }
}