use std::vec;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Voxel {
    id: u32,
}
impl Voxel {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Clone)]
/// Chunk consisting of blocks
pub struct Chunk {
    blocks: Vec<Voxel>,
    buffer: wgpu::Buffer,
}
impl Chunk {
    pub fn default(device: &wgpu::Device) -> Self {
        let blocks =
            vec![Voxel::new(0); Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * Self::CHUNK_HEIGHT];
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Buffer"),
            contents: bytemuck::cast_slice(&blocks),
            usage: wgpu::BufferUsages::STORAGE,
        });
        Self { blocks, buffer }
    }

    pub fn half(device: &wgpu::Device) -> Self {
        let block_amount = Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * Self::CHUNK_HEIGHT;
        let mut blocks = Vec::with_capacity(block_amount);

        for _ in 0..(block_amount / 2) {
            blocks.push(Voxel::new(1));
        }

        for _ in (block_amount / 2)..block_amount {
            blocks.push(Voxel::new(0));
        }

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Buffer"),
            contents: bytemuck::cast_slice(&blocks),
            usage: wgpu::BufferUsages::STORAGE,
        });

        Self { blocks, buffer }
    }

    const CHUNK_WIDTH: usize = 16;
    const CHUNK_LENGTH: usize = 16;
    const CHUNK_HEIGHT: usize = 32;

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + Self::CHUNK_WIDTH * (z + Self::CHUNK_LENGTH * y)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.blocks[Self::index(x, y, z)]
    }

    pub fn set(&mut self, block: Voxel, x: usize, y: usize, z: usize) {
        self.blocks[Self::index(x, y, z)] = block;
    }
}

/// World consisting of chunks
pub struct World {
    chunks: Vec<Chunk>,
}
impl World {
    pub fn default(device: &wgpu::Device) -> Self {
        let chunks = vec![Chunk::half(device); Self::WORLD_WIDTH * Self::WORLD_LENGTH];
        Self { chunks }
    }

    const WORLD_WIDTH: usize = 16;
    const WORLD_LENGTH: usize = 16;
}
