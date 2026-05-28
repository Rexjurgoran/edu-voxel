use noise::{NoiseFn, Perlin};
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
    pub(crate) face_bind_group: wgpu::BindGroup,
    pub face_count: u32,
}
impl Chunk {
    // -- Constants --
    const CHUNK_WIDTH: usize = 16;
    const CHUNK_LENGTH: usize = 16;
    const CHUNK_HEIGHT: usize = 32;
    const MIN_HEIGHT: f64 = 8.0;
    const MAX_HEIGHT: f64 = 20.0;
    const SEED: u32 = 1234;

    pub fn generate_voxels(chunk_x: usize, chunk_z: usize) -> Vec<Voxel> {
        let perlin = Perlin::new(Self::SEED);
        let mut voxels =
            vec![Voxel::new(0); Self::CHUNK_WIDTH * Self::CHUNK_LENGTH * Self::CHUNK_HEIGHT];

        for x in 0..Self::CHUNK_WIDTH {
            for z in 0..Self::CHUNK_LENGTH {
                // Convert to world coordinates
                let world_x = (chunk_x * Self::CHUNK_WIDTH + x) as f64;
                let world_z = (chunk_z * Self::CHUNK_LENGTH + z) as f64;

                // Sample noise - scale controls terrain frequency
                let scale = 0.02;
                let noise_val = perlin.get([world_x * scale, world_z * scale]);

                // Map noise from [-1, 1] to a block height
                let height =
                    ((noise_val + 1.0) / 2.0 * Self::MAX_HEIGHT + Self::MIN_HEIGHT) as usize;
                let height = height.clamp(1, Self::CHUNK_HEIGHT - 1);

                // Fill colum
                for y in 0..height {
                    let blocks = if y == height - 1 {
                        2 // Grass
                    } else if y > height - 5 {
                        1 // Dirt
                    } else {
                        3 // Stone
                    };
                    voxels[Self::index(x, y, z)] = Voxel::new(blocks);
                }
            }
        }
        voxels
    }

    pub fn from_voxels(
        device: &wgpu::Device,
        voxels: Vec<Voxel>,
        layout: &wgpu::BindGroupLayout,
        neighbors: &[Option<&Vec<Voxel>>; 6],
    ) -> Self {
        let faces = Self::get_faces(&voxels, neighbors);
        let face_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Face Buffer of Chunk"),
            contents: bytemuck::cast_slice(&faces),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let face_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: face_buffer.as_entire_binding(),
            }],
            label: Some("face_bind_group"),
        });

        Self {
            face_count: faces.len() as u32,
            face_bind_group,
        }
    }

    /// Get faces from voxels and neighboring chunks. Only faces that are adjacent to air will be returned.
    fn get_faces(voxels: &Vec<Voxel>, neighbors: &[Option<&Vec<Voxel>>; 6]) -> Vec<Face> {
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

                    // Face Left
                    if x == 0 {
                        // Voxel is left chunk border
                        let is_air = neighbors[1]
                            .map(|voxels| voxels[Self::index(Self::CHUNK_WIDTH - 1, y, z)].id == 0)
                            .unwrap_or(true); // If there is no neighboring chunk, treat it as air
                        if is_air {
                            faces.push(Face::new(location, FACE_LEFT, voxel_id));
                        }
                    } else if voxels[Self::index(x - 1, y, z)].id == 0 {
                        // Voxel to the left is air
                        faces.push(Face::new(location, FACE_LEFT, voxel_id));
                    }

                    // Face Right
                    if x == Self::CHUNK_WIDTH - 1 {
                        // Voxel is right chunk border
                        let is_air = neighbors[0]
                            .map(|voxels| voxels[Self::index(0, y, z)].id == 0)
                            .unwrap_or(true); // If there is no neighboring chunk, treat it as air
                        if is_air {
                            faces.push(Face::new(location, FACE_RIGHT, voxel_id));
                        }
                    } else if voxels[Self::index(x + 1, y, z)].id == 0 {
                        // Voxel to the right is air
                        faces.push(Face::new(location, FACE_RIGHT, voxel_id));
                    }

                    // Face Bottom
                    if y == 0 {
                        // Voxel is bottom chunk border
                        let is_air = neighbors[3]
                            .map(|voxels| voxels[Self::index(x, Self::CHUNK_HEIGHT - 1, z)].id == 0)
                            .unwrap_or(true); // If there is no neighboring chunk, treat it as air
                        if is_air {
                            faces.push(Face::new(location, FACE_BOTTOM, voxel_id));
                        }
                    } else if voxels[Self::index(x, y - 1, z)].id == 0 {
                        // Voxel below is air
                        faces.push(Face::new(location, FACE_BOTTOM, voxel_id));
                    }

                    // Face Top
                    if y == Self::CHUNK_HEIGHT - 1 {
                        // Voxel is top chunk border
                        let is_air = neighbors[2]
                            .map(|voxels| voxels[Self::index(x, 0, z)].id == 0)
                            .unwrap_or(true); // If there is no neighboring chunk, treat it as air
                        if is_air {
                            faces.push(Face::new(location, FACE_TOP, voxel_id));
                        }
                    } else if voxels[Self::index(x, y + 1, z)].id == 0 {
                        // Voxel above is air
                        faces.push(Face::new(location, FACE_TOP, voxel_id));
                    }

                    // Face Back
                    if z == 0 {
                        // Voxel is back chunk border
                        let is_air = neighbors[5]
                            .map(|voxels| voxels[Self::index(x, y, Self::CHUNK_LENGTH - 1)].id == 0)
                            .unwrap_or(true); // If there is no neighboring chunk, treat it as air
                        if is_air {
                            faces.push(Face::new(location, FACE_BACK, voxel_id));
                        }
                    } else if voxels[Self::index(x, y, z - 1)].id == 0 {
                        faces.push(Face::new(location, FACE_BACK, voxel_id));
                    }

                    // Face Front
                    if z == Self::CHUNK_LENGTH - 1 {
                        // Voxel is front chunk border
                        let is_air = neighbors[4]
                            .map(|voxels| voxels[Self::index(x, y, 0)].id == 0)
                            .unwrap_or(true); // If there is no neighboring chunk, treat it as air
                        if is_air {
                            faces.push(Face::new(location, FACE_FRONT, voxel_id));
                        }
                    } else if voxels[Self::index(x, y, z + 1)].id == 0 {
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
}

/// World consisting of chunks
pub struct World {
    pub(crate) chunks: Vec<Chunk>,
}
impl World {
    pub fn new(device: &wgpu::Device, face_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let mut chunks = Vec::new();
        let mut voxel_data = Vec::new();
        for z in 0..Self::WORLD_WIDTH {
            for x in 0..Self::WORLD_LENGTH {
                let chunk_voxels = Chunk::generate_voxels(x, z);
                voxel_data.push(chunk_voxels);
            }
        }

        // Create chunks from voxel data with neighboring chunk information
        for z in 0..Self::WORLD_WIDTH {
            for x in 0..Self::WORLD_LENGTH {
                let idx = Self::index(x, z);
                let neighbors = [
                    // Right neighbor
                    if x < Self::WORLD_WIDTH - 1 {
                        Some(&voxel_data[Self::index(x + 1, z)])
                    } else {
                        None
                    },
                    // Left neighbor
                    if x > 0 {
                        Some(&voxel_data[Self::index(x - 1, z)])
                    } else {
                        None
                    },
                    None, // Top neighbor
                    None, // Bottom neighbor
                    // Front neighbor
                    if z < Self::WORLD_LENGTH - 1 {
                        Some(&voxel_data[Self::index(x, z + 1)])
                    } else {
                        None
                    },
                    // Back neighbor
                    if z > 0 {
                        Some(&voxel_data[Self::index(x, z - 1)])
                    } else {
                        None
                    },
                ];
                let chunk = Chunk::from_voxels(
                    device,
                    voxel_data[idx].clone(),
                    face_bind_group_layout,
                    &neighbors,
                );
                chunks.push(chunk);
            }
        }
        Self { chunks }
    }

    fn index(x: usize, z: usize) -> usize {
        x + Self::WORLD_WIDTH * z
    }

    pub const WORLD_WIDTH: usize = 16;
    pub const WORLD_LENGTH: usize = 16;
}
