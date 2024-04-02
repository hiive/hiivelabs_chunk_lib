use crate::prelude::ChunkManager;
///
/// This trait must be implemented by your datasource.
/// It is consumed by the [`ChunkManager<T>::new()`] method.
pub trait TileMapDataSource<T> {
    /// Returns the width of the tilemap.
    ///
    /// returns: [`usize`]
    fn width(&self) -> usize;

    /// Returns the height of the tilemap.
    ///
    /// returns: [`usize`]
    fn height(&self) -> usize;

    /// Gets the index for the specified `(x, y)` coordinates.
    /// Returns an `Option<usize>` to handle out-of-bounds access gracefully.
    ///
    /// # Arguments
    ///
    /// * `x`: The _x_ coordinate.
    /// * `y`: The _y_ coordinate.
    ///
    ///
    /// returns: [`Option<usize>`]
    fn get_index_of(&self, x: usize, y: usize) -> Option<usize>;

    /// Returns the entire data of the tilemap as a `Vec<T>` for consumption.
    /// This might involve flattening the tilemap structure into a `Vec<T>`,
    /// depending on how the trait functionality is implemented.
    ///
    /// returns: [`Vec<T>`] of size `width * height`
    fn take_data(&mut self) -> Vec<T>;

    ///
    /// This returns the index of instance of `T` that is used
    /// if the procedural generator needs to access a value outside the map data.
    /// For example, if your tilemap is surrounded by sea, this would return the index of a default
    /// sea tile.
    ///
    /// returns: [`usize`]
    fn get_default_out_of_bounds_value_index(&self) -> usize;
}
