#[derive(Clone, Copy)]
/// Type of block
pub enum BlockType {
    Air,
    Grass,
}

#[derive(Clone)]
/// Chunk consisting of blocks
pub struct Chunk {
    blocks: Vec<BlockType>,
}
impl Chunk {
    pub fn default() -> Self {
        let blocks =
            vec![BlockType::Air; Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * Self::CHUNK_HEIGHT];
        Self { blocks }
    }

    const CHUNK_WIDTH: usize = 16;
    const CHUNK_LENGTH: usize = 16;
    const CHUNK_HEIGHT: usize = 256;

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + Self::CHUNK_WIDTH * (z + Self::CHUNK_LENGTH * y)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks[Self::index(x, y, z)]
    }

    pub fn set(&mut self, block: BlockType, x: usize, y: usize, z: usize) {
        self.blocks[Self::index(x, y, z)] = block;
    }
}

/// World consisting of chunks
pub struct World {
    chunks: Vec<Chunk>,
}
impl World {
    pub fn default() -> Self {
        let chunks = vec![Chunk::default(); Self::WORLD_WIDTH * Self::WORLD_LENGTH];
        Self { chunks }
    }

    const WORLD_WIDTH: usize = 16;
    const WORLD_LENGTH: usize = 16;
}
