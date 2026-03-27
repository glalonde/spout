// Pixel-perfect ship-terrain collision detection.
// Rasterizes the ship's triangle hull, tests every covered cell against
// the GPU terrain buffer.

struct CollisionUniforms {
    // Ship position in world coordinates.
    ship_x: f32,
    ship_y: f32,
    // Ship orientation in radians.
    ship_orientation: f32,
    // Terrain buffer coordinate offset (world Y of buffer row 0).
    terrain_buffer_offset: i32,
    // Terrain dimensions.
    terrain_width: u32,
    terrain_buffer_height: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: CollisionUniforms;

@group(0) @binding(1)
var<storage, read> terrain_buffer: array<i32>;

@group(0) @binding(2)
var<storage, read_write> result: array<u32>;

// Ship hull vertices in local space (matches ship.wgsl).
// Outer triangle: nose, left wing, right wing.
const NOSE: vec2<f32> = vec2<f32>(12.0, 0.0);
const LEFT_WING: vec2<f32> = vec2<f32>(-8.0, 9.0);
const RIGHT_WING: vec2<f32> = vec2<f32>(-8.0, -9.0);

fn rotate(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(c * v.x - s * v.y, s * v.x + c * v.y);
}

// Sign of the cross product (p2-p1) × (p-p1). Positive if p is to the left
// of the edge p1→p2.
fn edge_sign(p: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>) -> f32 {
    return (p2.x - p1.x) * (p.y - p1.y) - (p2.y - p1.y) * (p.x - p1.x);
}

// Point-in-triangle test using barycentric signs.
fn in_triangle(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> bool {
    let d1 = edge_sign(p, a, b);
    let d2 = edge_sign(p, b, c);
    let d3 = edge_sign(p, c, a);
    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
    return !(has_neg && has_pos);
}

fn is_solid(wx: i32, wy: i32) -> bool {
    if (wx < 0 || wx >= i32(uniforms.terrain_width)) {
        return false;
    }
    let row = wy - uniforms.terrain_buffer_offset;
    if (row < 0 || row >= i32(uniforms.terrain_buffer_height)) {
        return false;
    }
    let index = row * i32(uniforms.terrain_width) + wx;
    return terrain_buffer[index] > 0;
}

@compute @workgroup_size(1)
fn main() {
    // Transform hull vertices to world space.
    let a = rotate(NOSE, uniforms.ship_orientation) + vec2<f32>(uniforms.ship_x, uniforms.ship_y);
    let b = rotate(LEFT_WING, uniforms.ship_orientation) + vec2<f32>(uniforms.ship_x, uniforms.ship_y);
    let c = rotate(RIGHT_WING, uniforms.ship_orientation) + vec2<f32>(uniforms.ship_x, uniforms.ship_y);

    // Compute integer bounding box of the triangle.
    let min_x = i32(floor(min(min(a.x, b.x), c.x)));
    let max_x = i32(ceil(max(max(a.x, b.x), c.x)));
    let min_y = i32(floor(min(min(a.y, b.y), c.y)));
    let max_y = i32(ceil(max(max(a.y, b.y), c.y)));

    var hit = false;

    for (var y = min_y; y <= max_y; y = y + 1) {
        for (var x = min_x; x <= max_x; x = x + 1) {
            // Test cell center against the triangle.
            let p = vec2<f32>(f32(x) + 0.5, f32(y) + 0.5);
            if (in_triangle(p, a, b, c) && is_solid(x, y)) {
                hit = true;
            }
        }
    }

    result[0] = select(0u, 1u, hit);
}
