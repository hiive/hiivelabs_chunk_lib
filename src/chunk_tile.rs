// #[derive(Debug)]

pub struct ChunkTile<'t, T> {
    #[cfg(debug_assertions)]
    pub x: isize,
    #[cfg(debug_assertions)]
    pub y: isize,
    pub value: &'t T,
}

impl<'t, T> ChunkTile<'t, T> {
    #[cfg(debug_assertions)]
    pub(crate) fn new(x: isize, y: isize, value: &'t T) -> Self {
        Self { x, y, value }
    }

    #[cfg(not(debug_assertions))]
    pub(crate) fn new(value: &T) -> Self {
        Self { value }
    }
}
