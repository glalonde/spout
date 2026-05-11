// Ship-terrain collision detection with contact normal.
//
// Parallelizes over sampled hull points. Each workgroup lane Bresenham-walks
// one sampled point from the previous position to the current position, then
// a workgroup reduction picks the earliest hit.
//
// Result buffer layout:
//   [0] = hit (0 or 1)
//   [1] = normal_x (as u32 bits of f32)
//   [2] = normal_y (as u32 bits of f32)
//   [3] = impact_t (as u32 bits of f32; 0.0 previous state, 1.0 current state)

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

struct HitInfo {
    normal: vec2<f32>,
    t: f32,
};

const WORKGROUP_SIZE: u32 = 32u;

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
// Values are floor(edge length / 2.0), matching the original serial sampler.
const EDGE_STEPS: array<u32, 4> = array<u32, 4>(10u, 4u, 4u, 10u);
const NUM_COLLISION_SAMPLES: u32 = 28u;

var<workgroup> sample_t: array<f32, 32>;
var<workgroup> sample_normal: array<vec2<f32>, 32>;

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

fn no_hit() -> HitInfo {
    // A real collision at the final endpoint can have t=1.0. Keep no-hit
    // beyond the valid segment range so reduction still picks endpoint hits.
    return HitInfo(vec2<f32>(0.0, 0.0), 2.0);
}

fn fallback_normal(start: vec2<f32>, end: vec2<f32>) -> vec2<f32> {
    let delta = end - start;
    if (abs(delta.x) > abs(delta.y)) {
        if (delta.x >= 0.0) {
            return vec2<f32>(-1.0, 0.0);
        }
        return vec2<f32>(1.0, 0.0);
    }
    if (delta.y != 0.0) {
        if (delta.y >= 0.0) {
            return vec2<f32>(0.0, -1.0);
        }
        return vec2<f32>(0.0, 1.0);
    }
    return vec2<f32>(0.0, -1.0);
}

// Bresenham walk from `start` to `end`. Returns the normal of the first
// solid cell hit and the impact time, or no_hit() if no collision.
fn bresenham_check(start: vec2<f32>, end: vec2<f32>) -> HitInfo {
    let from_cell = vec2<i32>(floor(start));
    let to_cell = vec2<i32>(floor(end));
    let delta_i = to_cell - from_cell;
    let num_steps = abs(delta_i.x) + abs(delta_i.y);

    if (is_solid(from_cell.x, from_cell.y)) {
        return HitInfo(fallback_normal(start, end), 0.0);
    }

    if (num_steps == 0) {
        return no_hit();
    }

    let signed_delta = end - start;
    let delta = abs(signed_delta);
    let step = vec2<i32>(sign_i(signed_delta.x), sign_i(signed_delta.y));

    let start_remainder = (vec2<f32>(0.5) - (start - vec2<f32>(from_cell))) * vec2<f32>(step);
    var error = delta.x * start_remainder.y - delta.y * start_remainder.x;
    var cell = from_cell;

    for (var i = 0; i < num_steps; i = i + 1) {
        let old_cell = cell;
        let err_h = error - delta.y;
        let err_v = error + delta.x;

        if (err_v > -err_h) {
            // Horizontal step.
            error = err_h;
            cell.x = cell.x + step.x;
            if (is_solid(cell.x, cell.y)) {
                let boundary = select(f32(old_cell.x), f32(old_cell.x + 1), step.x > 0);
                let t = clamp((boundary - start.x) / signed_delta.x, 0.0, 1.0);
                return HitInfo(vec2<f32>(f32(-step.x), 0.0), t);
            }
        } else {
            // Vertical step.
            error = err_v;
            cell.y = cell.y + step.y;
            if (is_solid(cell.x, cell.y)) {
                let boundary = select(f32(old_cell.y), f32(old_cell.y + 1), step.y > 0);
                let t = clamp((boundary - start.y) / signed_delta.y, 0.0, 1.0);
                return HitInfo(vec2<f32>(0.0, f32(-step.y)), t);
            }
        }
    }

    return no_hit();
}

// Sweep a single local-space point from its previous world position to its
// current world position, returning hit info if it collides.
fn check_point(local_pos: vec2<f32>) -> HitInfo {
    let prev_world = rotate(local_pos, uniforms.prev_ship_orientation)
        + vec2<f32>(uniforms.prev_ship_x, uniforms.prev_ship_y);
    let curr_world = rotate(local_pos, uniforms.ship_orientation)
        + vec2<f32>(uniforms.ship_x, uniforms.ship_y);
    return bresenham_check(prev_world, curr_world);
}

fn edge_sample(edge_index: u32, step_index: u32) -> vec2<f32> {
    let a = HULL_VERTS[edge_index];
    let b = HULL_VERTS[(edge_index + 1u) % NUM_HULL_VERTS];
    let t = f32(step_index) / f32(EDGE_STEPS[edge_index]);
    return a + (b - a) * t;
}

fn sample_point(sample_index: u32) -> vec2<f32> {
    if (sample_index < NUM_HULL_VERTS) {
        return HULL_VERTS[sample_index];
    }

    var remaining = sample_index - NUM_HULL_VERTS;
    for (var edge = 0u; edge < NUM_HULL_VERTS; edge = edge + 1u) {
        let interior_samples = EDGE_STEPS[edge] - 1u;
        if (remaining < interior_samples) {
            return edge_sample(edge, remaining + 1u);
        }
        remaining = remaining - interior_samples;
    }

    return HULL_VERTS[0];
}

@compute @workgroup_size(32)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    let lane = local_id.x;
    var lane_hit = no_hit();

    if (lane < NUM_COLLISION_SAMPLES) {
        lane_hit = check_point(sample_point(lane));
    }

    sample_t[lane] = lane_hit.t;
    sample_normal[lane] = lane_hit.normal;
    workgroupBarrier();

    var stride = WORKGROUP_SIZE / 2u;
    loop {
        if (lane < stride) {
            let other = lane + stride;
            if (sample_t[other] < sample_t[lane]) {
                sample_t[lane] = sample_t[other];
                sample_normal[lane] = sample_normal[other];
            }
        }
        workgroupBarrier();

        if (stride == 1u) {
            break;
        }
        stride = stride / 2u;
    }

    if (lane == 0u) {
        let best_normal = sample_normal[0];
        let hit = best_normal.x != 0.0 || best_normal.y != 0.0;
        result[0] = select(0u, 1u, hit);
        result[1] = bitcast<u32>(best_normal.x);
        result[2] = bitcast<u32>(best_normal.y);
        result[3] = bitcast<u32>(sample_t[0]);
    }
}
