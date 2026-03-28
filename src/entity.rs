use cgmath::Vector3;
use wgpu::util::DeviceExt;

pub const FACE_TOP: u8 = 1;
pub const FACE_FRONT: u8 = 2;
pub const FACE_RIGHT: u8 = 3;
pub const FACE_LEFT: u8 = 4;
pub const FACE_BACK: u8 = 5;
pub const FACE_BOTTOM: u8 = 6;

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
    // Position of the bottom left corner of the face
    location: [u8; 3],
    direction: u8,
}

/// Chunk consisting of blocks
pub struct Chunk {
    blocks: Vec<Voxel>,
    // Faces I want to render
    face_buffer: wgpu::Buffer,
    // Make vertices reusable
    index_buffer: wgpu::Buffer,
    pub face_count: u32,
}
impl Chunk {
    // -- Constants --
    const CHUNK_WIDTH: usize = 16;
    const CHUNK_LENGTH: usize = 16;
    const CHUNK_HEIGHT: usize = 32;

    // Create a new chunk with all blocks set to 0
    pub fn new(device: &wgpu::Device) -> Self {
        let blocks =
            vec![Voxel::new(0); Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * Self::CHUNK_HEIGHT];

        let faces = Self::get_faces();
        let face_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Face Buffer of Chunk"),
            contents: bytemuck::cast_slice(&faces),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let indeces: Vec<u32> = vec![0, 1, 2, 2, 3, 0];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer of Chunk"),
            contents: bytemuck::cast_slice(&indeces),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            blocks,
            face_buffer,
            index_buffer,
            face_count: faces.len() as u32,
        }
    }

    // Create a new chunk with half of the blocks set to 1 and the other half set to 0
    pub fn half(device: &wgpu::Device) -> Self {
        let block_amount = Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * Self::CHUNK_HEIGHT;
        let mut blocks = Vec::with_capacity(block_amount);

        for _ in 0..(block_amount / 2) {
            blocks.push(Voxel::new(1));
        }

        for _ in (block_amount / 2)..block_amount {
            blocks.push(Voxel::new(0));
        }

        let faces = Self::get_faces();
        let face_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Face Buffer of Chunk"),
            contents: bytemuck::cast_slice(&faces),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let indeces: Vec<u32> = vec![0, 1, 2, 2, 3, 0];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer of Chunk"),
            contents: bytemuck::cast_slice(&indeces),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            blocks,
            face_buffer,
            index_buffer,
            face_count: faces.len() as u32,
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
                    direction: FACE_TOP,
                });
            }
        }
        faces
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + Self::CHUNK_WIDTH * (y + Self::CHUNK_LENGTH * z)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.blocks[Self::index(x, y, z)]
    }

    pub fn set(&mut self, block: Voxel, x: usize, y: usize, z: usize) {
        self.blocks[Self::index(x, y, z)] = block;
    }

    pub fn get_face_buffer(&self) -> &wgpu::Buffer {
        &self.face_buffer
    }
}

/// World consisting of chunks
pub struct World {
    chunks: Vec<Chunk>,
}
impl World {
    pub fn new(device: &wgpu::Device) -> Self {
        let chunks = (0..Self::WORLD_WIDTH * Self::WORLD_LENGTH)
            .map(|_| Chunk::half(device))
            .collect();
        Self { chunks }
    }

    const WORLD_WIDTH: usize = 16;
    const WORLD_LENGTH: usize = 16;
}
