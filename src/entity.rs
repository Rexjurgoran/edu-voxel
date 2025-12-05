use cgmath::Vector3;
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

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Face {
    location: Vector3<u8>,
    direction: u8,
}

#[derive(Clone)]
/// Chunk consisting of blocks
pub struct Chunk {
    blocks: Vec<Voxel>,
    // Faces I want to render
    face_buffer: wgpu::Buffer,
    // Make vertices reusable
    index_buffer: wgpu::Buffer,
}
impl Chunk {
    pub fn default(device: &wgpu::Device) -> Self {
        let blocks =
            vec![Voxel::new(0); Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * Self::CHUNK_HEIGHT];
        let face_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Face Buffer of Chunk"),
            contents: bytemuck::cast_slice(&Self::get_faces()),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer of Chunk"),
            contents: bytemuck::cast_slice(&indeces),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            blocks,
            face_buffer,
            index_buffer,
        }
    }

    fn get_faces() -> Vec<Face> {
        let size = Self::CHUNK_WIDTH * Self::CHUNK_LENGTH;
        let mut faces = Vec::with_capacity(size);
        for x in 0..Self::CHUNK_WIDTH {
            for y in 0..Self::CHUNK_LENGTH {
                faces.push(Face {
                    location: Vector3::new(x as u8, y as u8, 15),
                    direction: 1,
                });
            }
        }
        faces
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
