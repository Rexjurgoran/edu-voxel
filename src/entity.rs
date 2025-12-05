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

/// Single face of a Voxel. Will be converted to 4 vertices / 2 triangles in
/// shader. Direction is 1-6, like a physical dice.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Face {
    location: [u8; 3],
    direction: u8,
}

impl Face {
    pub const VERTICES: usize = 4;
    pub const INDECES: usize = 6;
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
    const CHUNK_WIDTH: usize = 16;
    const CHUNK_LENGTH: usize = 16;
    const CHUNK_HEIGHT: usize = 32;

    pub fn default(device: &wgpu::Device) -> Self {
        let blocks =
            vec![Voxel::new(0); Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * Self::CHUNK_HEIGHT];
        let faces = Self::get_faces();
        let face_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Face Buffer of Chunk"),
            contents: bytemuck::cast_slice(&faces),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer of Chunk"),
            contents: bytemuck::cast_slice(&Self::get_indeces(faces.len())),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            blocks,
            face_buffer,
            index_buffer,
        }
    }

    /// Get faces. First only default implementation, later generate from
    /// blocks
    fn get_faces() -> Vec<Face> {
        let size = Self::CHUNK_WIDTH * Self::CHUNK_LENGTH;
        let mut faces = Vec::with_capacity(size);
        for x in 0..Self::CHUNK_WIDTH {
            for y in 0..Self::CHUNK_LENGTH {
                faces.push(Face {
                    location: [x as u8, y as u8, 15],
                    direction: 1,
                });
            }
        }
        faces
    }

    /// Create index vector out of size of face vector
    /// 2 Faces result in [0,1,2,2,3,0,4,5,6,6,7,4]
    fn get_indeces(size: usize) -> Vec<usize> {
        let mut indeces = Vec::with_capacity(size * Face::INDECES);
        for i in 0..size {
            let offset = i * Face::VERTICES;
            let mut new = vec![
                offset,
                offset + 1,
                offset + 2,
                offset + 2,
                offset + 3,
                offset,
            ];
            indeces.append(&mut new);
        }
        indeces
    }

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
        let chunks = vec![Chunk::default(device); Self::WORLD_WIDTH * Self::WORLD_LENGTH];
        Self { chunks }
    }

    const WORLD_WIDTH: usize = 16;
    const WORLD_LENGTH: usize = 16;
}
