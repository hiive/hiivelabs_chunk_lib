use crate::chunk::Chunk;

pub(crate) enum ChunkStorageMessage {
    ToStore(Chunk),
    ShutDown,
}
