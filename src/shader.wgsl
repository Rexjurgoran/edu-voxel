// Constants
const TILES_PER_ROW: f32 = 2.0;
const TILES_PER_COLUMN: f32 = 2.0;
const TILE_WIDTH: f32 = 1.0 / TILES_PER_ROW;
const TILE_HEIGHT: f32 = 1.0 / TILES_PER_COLUMN;
const NORMALS = array<vec3<f32>, 6>(
    vec3<f32>( 1, 0, 0),
    vec3<f32>(-1, 0, 0),
    vec3<f32>( 0, 1, 0),
    vec3<f32>( 0,-1, 0),
    vec3<f32>( 0, 0, 1),
    vec3<f32>( 0, 0,-1),
);
const TANGENTS = array<vec3<f32>, 6>(
    vec3<f32>(0,0,1),
    vec3<f32>(0,0,1),
    vec3<f32>(1,0,0),
    vec3<f32>(1,0,0),
    vec3<f32>(1,0,0),
    vec3<f32>(1,0,0),
);
const BITANGENTS = array<vec3<f32>, 6>(
    vec3<f32>(0,1,0),
    vec3<f32>(0,1,0),
    vec3<f32>(0,0,1),
    vec3<f32>(0,0,1),
    vec3<f32>(0,1,0),
    vec3<f32>(0,1,0),
);

// Structs
struct Camera {
    view_pos: vec4<f32>,
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
}
struct Face {
    data0: u32, // Packed data for location and direction
    data1: u32, // Block ID and padding for alignment
}
struct UnpackedFace {
    location: vec3<f32>,
    direction: u32,
    uv_offset: vec2<f32>, // top-left corner of the face in the texture atlas
}
@group(1) @binding(0)
var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(2) @binding(0)
var<uniform> light: Light;

@group(4) @binding(0)
var<storage, read> faces: array<Face>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_view_position: vec3<f32>,
    @location(3) world_normal: vec3<f32>,
    @location(4) world_tangent: vec3<f32>,
    @location(5) world_bitangent: vec3<f32>,
};

// Vertex shader runs once per vertex to determine the position of the vertex 
// on the screen and pass data to the fragment shader
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // By dividing the vertex index by the number of vertices per face, we can 
    // determine which face we are currently processing
    let face_index = vertex_index / 6u; 

    // Unpack face and match normals from direction
    let face = unpack_face(faces[face_index]);
    let normal = NORMALS[face.direction];
    let tangent = TANGENTS[face.direction];
    let bitangent = BITANGENTS[face.direction];

    // Get position of the vertex by adding the corner offset to the face location
    let corner_index = get_corner_index(vertex_index % 6u);
    let corner_offset = get_corner_offset(corner_index);

    // Get offset from the center of the voxel to the face plane
    // 0 for even directions (top, right, back), 1 for odd directions (bottom, left, front)
    let face_offset = select(0.0, 1.0, face.direction % 2u == 0u); 
    let position = face.location 
        + tangent * corner_offset.x
        + bitangent * corner_offset.y
        + normal * face_offset;

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
    out.tex_coords = face.uv_offset + vec2<f32>(corner_offset.x * TILE_WIDTH, corner_offset.y * TILE_HEIGHT);
    out.world_position = position;
    out.world_normal = normal;
    out.world_tangent = tangent;
    out.world_bitangent = bitangent;
    out.world_view_position = camera.view_pos.xyz;
    return out;
}

// Unpacks the face data from a single u32 into its components. This is more 
//memory efficient, because we just need to unpack the current face
fn unpack_face(face: Face) -> UnpackedFace {
    let x = f32(face.data0 & 0xFFu); // Extract the lower 8 bits
    let y = f32((face.data0 >> 8) & 0xFFu); // Extract the next 8 bits
    let z = f32((face.data0 >> 16) & 0xFFu); // Extract the next 8 bits
    let dir = u32((face.data0 >> 24) & 0xFFu);
    let block_id = u32(face.data1 & 0xFFu); // Extract the lower 8 bits for block ID
    let uv_offset = vec2<f32>(f32(
        block_id % 2u) * TILE_WIDTH, // Calculate the x offset in the texture atlas
        f32(block_id / 2u) * TILE_HEIGHT); // Calculate the y offset in the texture atlas
    return UnpackedFace(vec3<f32>(x, y, z), dir, uv_offset);
}

// Map the vertex index to the corresponding corner of the face
// 3 --- 2
// |   / |
// | /   |
// 0 --- 1
fn get_corner_index(face_vertex: u32) -> u32 {
    switch (face_vertex) {
        // First triangle
        case 0u: { return 0u; }
        case 1u: { return 1u; }
        case 2u: { return 2u; }
        // Second triangle
        case 3u: { return 2u; }
        case 4u: { return 3u; }
        default: { return 0u; }
    }
}

// Map the corner index to the corresponding offset on the face
fn get_corner_offset(corner_index: u32) -> vec2<f32> {
    switch (corner_index) {
        case 0u: { return vec2<f32>(0.0, 0.0); } // Bottom-left
        case 1u: { return vec2<f32>(1.0, 0.0); } // Bottom-right
        case 2u: { return vec2<f32>(1.0, 1.0); } // Top-right
        default: { return vec2<f32>(0.0, 1.0); } // Top-left
    }
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;
@group(0)@binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@group(3)
@binding(0)
var env_map: texture_cube<f32>;
@group(3)
@binding(1)
var env_sampler: sampler;

// Fragment shader runs once per pixel to determine the final color of the pixel
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    // Adjust the tangent and bitangent using the Gramm-Schmidt process
    // This makes sure that they are perpendicular to each other and the
    // normal of the surface.
    let world_tangent = normalize(in.world_tangent - dot(in.world_tangent, in.world_normal) * in.world_normal);
    let world_bitangent = cross(world_tangent, in.world_normal);
    
    // Convert the normal sample to world space
    let TBN = mat3x3(
        world_tangent,
        world_bitangent,
        in.world_normal,
    );
    let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    let world_normal = TBN * tangent_normal;

    // Create the lighting vectors
    let light_dir = normalize(light.position - in.world_position);
    let view_dir = normalize(in.world_view_position - in.world_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let specular_strength = pow(max(dot(world_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    // Calculate reflections
    let world_reflect = reflect(-view_dir, world_normal);
    let reflection = textureSample(env_map, env_sampler, world_reflect).rgb;
    let shininess = 0.1;

    let result = (diffuse_color + specular_color) * object_color.xyz + reflection * shininess;

    return vec4<f32>(result, object_color.a);
    //return vec4<f32>(world_normal, 1.0); // Visualize normals for debugging
}