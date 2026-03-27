// Ship-terrain collision detection compute shader.
// Checks a few points around the ship against the GPU terrain buffer
// and writes a collision flag (0 or 1) to the output buffer.

struct CollisionUniforms {
    // Ship position in world coordinates.
    ship_x: f32,
    ship_y: f32,
    // Terrain buffer coordinate offset (world Y of buffer row 0).
    terrain_buffer_offset: i32,
    // Terrain dimensions.
    terrain_width: u32,
    terrain_buffer_height: u32,
    // Ship collision radius in world units.
    collision_radius: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: CollisionUniforms;

@group(0) @binding(1)
var<storage, read> terrain_buffer: array<i32>;

@group(0) @binding(2)
var<storage, read_write> result: array<u32>;

fn check_point(wx: f32, wy: f32) -> bool {
    let xi = i32(wx);
    let yi = i32(wy);

    if (xi < 0 || xi >= i32(uniforms.terrain_width)) {
        return false;
    }

    let row = yi - uniforms.terrain_buffer_offset;
    if (row < 0 || row >= i32(uniforms.terrain_buffer_height)) {
        return false;
    }

    let index = row * i32(uniforms.terrain_width) + xi;
    return terrain_buffer[index] > 0;
}

@compute @workgroup_size(1)
fn main() {
    let cx = uniforms.ship_x;
    let cy = uniforms.ship_y;
    let r = uniforms.collision_radius;

    var hit = false;
    // Test center + 4 cardinal points.
    hit = hit || check_point(cx, cy);
    hit = hit || check_point(cx + r, cy);
    hit = hit || check_point(cx - r, cy);
    hit = hit || check_point(cx, cy + r);
    hit = hit || check_point(cx, cy - r);

    result[0] = select(0u, 1u, hit);
}
