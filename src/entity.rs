use wgpu::util::DeviceExt;

// Face direction constants. The origin of a voxel is the bottom left corner.
// The direction is used to calculate the position of the vertices of the face.
pub const FACE_RIGHT: u8 = 0; // positive X direction
pub const FACE_LEFT: u8 = 1; // negative X direction
pub const FACE_TOP: u8 = 2; // positive Y direction
pub const FACE_BOTTOM: u8 = 3; // negative Y direction
pub const FACE_FRONT: u8 = 4; // positive Z direction
pub const FACE_BACK: u8 = 5; // negative Z direction

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Voxel {
    id: u8,
}
impl Voxel {
    pub fn new(id: u8) -> Self {
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
    block_id: u8,
    _padding: [u8; 3], // Padding to make the struct 8 bytes long (2 x u32)
}

impl Face {
    pub fn new(location: [u8; 3], direction: u8, block_id: u8) -> Self {
        Self {
            location,
            direction,
            block_id,
            _padding: [0; 3],
        }
    }
}

/// Chunk consisting of blocks
pub struct Chunk {
    voxels: Vec<Voxel>,
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

        let faces = Self::get_faces(&blocks);
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
            voxels: blocks,
            face_buffer,
            index_buffer,
            face_count: faces.len() as u32,
        }
    }

    // Create a new chunk with half of the blocks set to 1 and the other half set to 0
    pub fn half(device: &wgpu::Device) -> Self {
        let block_amount = Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * Self::CHUNK_HEIGHT;
        let mut blocks = Vec::with_capacity(block_amount);

        println!(
            "Creating {} blocks of stone",
            Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * 10
        );
        for _ in 0..Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * 10 {
            blocks.push(Voxel::new(3));
        }

        println!(
            "Creating {} blocks of dirt",
            Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * 5
        );
        for _ in 0..Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * 5 {
            blocks.push(Voxel::new(1));
        }

        println!(
            "Creating {} blocks of grass",
            Self::CHUNK_WIDTH * Self::CHUNK_LENGTH
        );
        for _ in 0..Self::CHUNK_WIDTH * Self::CHUNK_LENGTH {
            blocks.push(Voxel::new(2));
        }

        print!(
            "Creating {} blocks of air",
            Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * 16
        );
        for _ in 0..Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * 16 {
            blocks.push(Voxel::new(0));
        }

        let faces = Self::get_faces(&blocks);
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
            voxels: blocks,
            face_buffer,
            index_buffer,
            face_count: faces.len() as u32,
        }
    }

    /// Get faces. First only default implementation, later generate from
    /// blocks
    fn get_faces(voxels: &Vec<Voxel>) -> Vec<Face> {
        let mut faces = Vec::new();
        for x in 0..Self::CHUNK_WIDTH {
            for y in 0..Self::CHUNK_HEIGHT {
                for z in 0..Self::CHUNK_LENGTH {
                    // If the block is air, skip it
                    let voxel_id = voxels[Self::index(x, y, z)].id;
                    if voxel_id == 0 {
                        continue;
                    }

                    let location = [x as u8, y as u8, z as u8];

                    if x == 0 || voxels[Self::index(x - 1, y, z)].id == 0 {
                        faces.push(Face::new(location, FACE_LEFT, voxel_id));
                    }

                    if x == Self::CHUNK_WIDTH - 1 || voxels[Self::index(x + 1, y, z)].id == 0 {
                        faces.push(Face::new(location, FACE_RIGHT, voxel_id));
                    }

                    if y == 0 || voxels[Self::index(x, y - 1, z)].id == 0 {
                        faces.push(Face::new(location, FACE_BOTTOM, voxel_id));
                    }

                    if y == Self::CHUNK_HEIGHT - 1 || voxels[Self::index(x, y + 1, z)].id == 0 {
                        faces.push(Face::new(location, FACE_TOP, voxel_id));
                    }

                    if z == 0 || voxels[Self::index(x, y, z - 1)].id == 0 {
                        faces.push(Face::new(location, FACE_BACK, voxel_id));
                    }

                    if z == Self::CHUNK_LENGTH - 1 || voxels[Self::index(x, y, z + 1)].id == 0 {
                        faces.push(Face::new(location, FACE_FRONT, voxel_id));
                    }
                }
            }
        }
        faces
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + Self::CHUNK_WIDTH * (z + Self::CHUNK_LENGTH * y)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels[Self::index(x, y, z)]
    }

    pub fn set(&mut self, block: Voxel, x: usize, y: usize, z: usize) {
        self.voxels[Self::index(x, y, z)] = block;
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
