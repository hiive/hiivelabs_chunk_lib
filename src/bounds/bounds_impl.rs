use bitcode::{Decode, Encode};
use hiivelabs_rand_utils_lib::prelude::{IDim, UDim};

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub(crate) struct Bounds {
    pub(crate) x: IDim,
    pub(crate) y: IDim,
    pub(crate) width: UDim,
    pub(crate) height: UDim,
    pub(crate) padding: UDim,
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
    pub(crate) fn get_index_for_coords(
        &self,
        x: IDim,
        y: IDim,
        err_on_out_of_bounds: bool,
    ) -> Result<(IDim, IDim, IDim), &str> {
        let padding = self.padding as IDim;
        let width = self.width as IDim;
        let height = self.height as IDim;
        let (adj_x, adj_y) = self.get_adjusted_coordinates(x, y);

        // #[cfg(debug_assertions)]
        // {
        //     let sx = self.x;
        //     let sy = self.y;
        //     println!("Bounds::get_index_for_coords: self:({sx}, {sy}) pad:({padding})");
        //     println!("Bounds::get_index_for_coords: ({x}, {y}) -> adj:({adj_x}, {adj_y})");
        //     println!();
        // }

        let padded_width = width + 2 * padding;
        let padded_height = height + 2 * padding;

        if err_on_out_of_bounds
            && ((adj_x < 0) || (adj_x >= padded_width) || (adj_y < 0) || (adj_y >= padded_height))
        {
            return Err("Chunk coordinates are out of bounds");
        }

        Ok((adj_y * padded_width + adj_x, adj_x, adj_y))
    }

    pub(crate) fn get_width(&self, include_padding: bool) -> UDim {
        if include_padding {
            self.width + self.padding * 2
        } else {
            self.width
        }
    }

    pub(crate) fn get_height(&self, include_padding: bool) -> UDim {
        if include_padding {
            self.height + self.padding * 2
        } else {
            self.height
        }
    }

    pub(crate) fn get_adjusted_coordinates(&self, x: IDim, y: IDim) -> (IDim, IDim) {
        let padding = self.padding as IDim;
        (x + padding - self.x, y + padding - self.y)
    }

    pub(crate) fn get_bound_coords(&self, include_padding: bool) -> (IDim, IDim, IDim, IDim) {
        let padding = if include_padding {
            self.padding as IDim
        } else {
            0
        };

        let padded_width = self.width as IDim + 2 * padding;
        let padded_height = self.height as IDim + 2 * padding;

        let min_x = self.x - padding;
        let min_y = self.y - padding;

        let max_x = min_x + padded_width;
        let max_y = min_y + padded_height;

        (min_x, min_y, max_x, max_y)
    }

    pub(crate) fn get_tile_count(&self, include_padding: bool) -> usize {
        let padding = if include_padding { self.padding } else { 0 };

        let padded_width = self.width + 2 * padding;
        let padded_height = self.height + 2 * padding;
        (padded_width * padded_height) as usize
    }

    pub(crate) fn is_coords_in_bounds(&self, tx: IDim, ty: IDim, with_padding: bool) -> bool {
        let (min_x, min_y, max_x, max_y) = self.get_bound_coords(with_padding);
        tx >= min_x && tx < max_x && ty >= min_y && ty < max_y
    }

    /// This confirms that the specified index is in bounds.
    pub(crate) fn is_index_in_bounds(&self, ix: IDim) -> bool {
        let padding = self.padding as IDim;
        let padded_width = self.width as IDim + 2 * padding;
        let padded_height = self.height as IDim + 2 * padding;
        let max_ix = padded_width * padded_height;
        // check is in bounds.
        ix >= 0 && ix < max_ix
    }
}
