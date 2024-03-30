///
/// This trait must be implemented by your datasource.
/// It is consumed by the [`crate::ChunkManager<T>::new()`] method.
pub trait TileMapDataSource<T> {
    /// Returns the width of the tilemap.
    fn width(&self) -> usize;

    /// Returns the height of the tilemap.
    fn height(&self) -> usize;

    /// Gets the index for the specified `(x, y)` coordinates.
    /// Returns an `Option<usize>` to handle out-of-bounds access gracefully.
    ///
    /// # Arguments
    ///
    /// * `x`: The _x_ coordinate.
    /// * `y`: The _y_ coordinate.
    fn get_index_of(&self, x: usize, y: usize) -> Option<usize>;

    /// Returns the entire data of the tilemap as a `Vec<T>` for consumption.
    /// This might involve flattening the tilemap structure into a `Vec<T>`,
    /// depending on how the trait functionality is implemented.
    fn take_data(&mut self) -> Vec<T>;
}
