pub trait TileMapDataSource<T> {
    // Returns the width of the tilemap.
    fn width(&self) -> usize;

    // Returns the height of the tilemap.
    fn height(&self) -> usize;

    // Gets the value at the specified coordinates.
    // Returns an Option<T> to handle out-of-bounds access gracefully.
    fn get_at(&self, x: usize, y: usize) -> Option<&T>;

    // Returns the entire data of the tilemap as a Vec<T>.
    // This might involve flattening the tilemap structure into a Vec.
    fn get_data(&self) -> Vec<&T>;

    /*
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_>;

    // implement with:
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.data.iter())
    }
    */
}
