// Ship-terrain collision detection with contact normal.
//
// For each hull vertex, Bresenham-walks from the previous position to the
// current position. If a solid cell is hit, records whether the step was
// horizontal or vertical to produce a bounce normal.
//
// Result buffer layout:
//   [0] = hit (0 or 1)
//   [1] = normal_x (as u32 bits of f32)
//   [2] = normal_y (as u32 bits of f32)

struct CollisionUniforms {
    // Current ship position.
    ship_x: f32,
    ship_y: f32,
    // Previous ship position.
    prev_ship_x: f32,
    prev_ship_y: f32,
    // Ship orientation (current and previous).
    ship_orientation: f32,
    prev_ship_orientation: f32,
    // Terrain buffer.
    terrain_buffer_offset: i32,
    terrain_width: u32,
    terrain_buffer_height: u32,
    _pad: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: CollisionUniforms;

@group(0) @binding(1)
var<storage, read> terrain_buffer: array<i32>;

@group(0) @binding(2)
var<storage, read_write> result: array<u32>;

// Ship hull vertices in local space (matches ship.wgsl outline_vertices).
// Must stay in sync with the 4-vertex perimeter in ship.wgsl.
const HULL_VERTS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>( 12.0,  0.0),  // nose
    vec2<f32>( -8.0,  9.0),  // left wing
    vec2<f32>( -5.0,  0.0),  // tail notch
    vec2<f32>( -8.0, -9.0),  // right wing
);

const NUM_HULL_VERTS: u32 = 4u;
// Sample hull edges every ~2 world units so no 1-cell gap escapes detection.
const SAMPLE_STEP: f32 = 2.0;

fn rotate(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(c * v.x - s * v.y, s * v.x + c * v.y);
}

fn is_solid(wx: i32, wy: i32) -> bool {
    if (wx < 0 || wx >= i32(uniforms.terrain_width)) {
        return false;
    }
    let row = wy - uniforms.terrain_buffer_offset;
    if (row < 0 || row >= i32(uniforms.terrain_buffer_height)) {
        return false;
    }
    return terrain_buffer[row * i32(uniforms.terrain_width) + wx] > 0;
}

fn sign_i(v: f32) -> i32 {
    if (v >= 0.0) { return 1; } else { return -1; }
}

// Bresenham walk from `start` to `end`. Returns the normal of the first
// solid cell hit, or (0,0) if no collision.
fn bresenham_check(start: vec2<f32>, end: vec2<f32>) -> vec2<f32> {
    let from_cell = vec2<i32>(floor(start));
    let to_cell = vec2<i32>(floor(end));
    let delta_i = to_cell - from_cell;
    let num_steps = abs(delta_i.x) + abs(delta_i.y);

    if (num_steps == 0) {
        // Didn't move cells — just check current cell.
        if (is_solid(to_cell.x, to_cell.y)) {
            return normalize(start - end);
        }
        return vec2<f32>(0.0, 0.0);
    }

    let signed_delta = end - start;
    let delta = abs(signed_delta);
    let step = vec2<i32>(sign_i(signed_delta.x), sign_i(signed_delta.y));

    let start_remainder = (vec2<f32>(0.5) - (start - vec2<f32>(from_cell))) * vec2<f32>(step);
    var error = delta.x * start_remainder.y - delta.y * start_remainder.x;
    var cell = from_cell;

    for (var i = 0; i < num_steps; i = i + 1) {
        let err_h = error - delta.y;
        let err_v = error + delta.x;

        if (err_v > -err_h) {
            // Horizontal step.
            error = err_h;
            cell.x = cell.x + step.x;
            if (is_solid(cell.x, cell.y)) {
                return vec2<f32>(f32(-step.x), 0.0);
            }
        } else {
            // Vertical step.
            error = err_v;
            cell.y = cell.y + step.y;
            if (is_solid(cell.x, cell.y)) {
                return vec2<f32>(0.0, f32(-step.y));
            }
        }
    }

    return vec2<f32>(0.0, 0.0);
}

// Sweep a single local-space point from its previous world position to its
// current world position, returning the contact normal (or (0,0) if clear).
fn check_point(local_pos: vec2<f32>) -> vec2<f32> {
    let prev_world = rotate(local_pos, uniforms.prev_ship_orientation)
        + vec2<f32>(uniforms.prev_ship_x, uniforms.prev_ship_y);
    let curr_world = rotate(local_pos, uniforms.ship_orientation)
        + vec2<f32>(uniforms.ship_x, uniforms.ship_y);
    return bresenham_check(prev_world, curr_world);
}

@compute @workgroup_size(1)
fn main() {
    var best_normal = vec2<f32>(0.0, 0.0);

    // Check each hull vertex.
    for (var i = 0u; i < NUM_HULL_VERTS; i = i + 1u) {
        let n = check_point(HULL_VERTS[i]);
        if (n.x != 0.0 || n.y != 0.0) {
            best_normal = n;
        }
    }

    // Sample each hull edge at ~SAMPLE_STEP world-unit intervals.
    // This fills the gaps between vertices so thin obstacles aren't missed.
    for (var i = 0u; i < NUM_HULL_VERTS; i = i + 1u) {
        let j = (i + 1u) % NUM_HULL_VERTS;
        let a = HULL_VERTS[i];
        let b = HULL_VERTS[j];
        let edge = b - a;
        let steps = max(1u, u32(length(edge) / SAMPLE_STEP));
        for (var s = 1u; s < steps; s = s + 1u) {
            let t = f32(s) / f32(steps);
            let n = check_point(a + edge * t);
            if (n.x != 0.0 || n.y != 0.0) {
                best_normal = n;
            }
        }
    }

    let hit = best_normal.x != 0.0 || best_normal.y != 0.0;
    result[0] = select(0u, 1u, hit);
    result[1] = bitcast<u32>(best_normal.x);
    result[2] = bitcast<u32>(best_normal.y);
}
