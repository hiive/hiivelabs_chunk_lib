
/*
use chunk_lib::Chunk;
use chunk_lib::ChunkLayer;
use chunk_lib::ChunkTile;

#[test]
fn test_chunk_export_from_lib() {
    let mut chunk = Chunk::<i32>::new(0, 0, 10, 10, 1);
    chunk.set_at(0, 0, &1);
    let c = chunk.get_at(0, 0);
    assert!(c.is_some() && c.unwrap() == &1);
    let c = chunk.get_at(1, 1);
    assert!(c.is_none());
}

#[test]
fn test_set_up_chunk_layer() {
    let mut chunk_layer = ChunkLayer::<i32>::new(1, 10, 10, 1, 10, 10);
    chunk_layer.set_at(0, 0, &1);
    let test = chunk_layer.get_at(0, 0);
    assert_eq!(test.unwrap(), &1);
}
*/