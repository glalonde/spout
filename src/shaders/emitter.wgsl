{% include "particle.wgsl.include" %}
{% include "grid.wgsl.include" %}

let PI: f32 = 3.14159265358979323846;

// Size 32, Alignment 8, no padding.
struct EmitterMotion {
    position_start: vec2<f32>;
    position_end: vec2<f32>;
    velocity: vec2<f32>;
    angle_start: f32;
    angle_end: f32;
};

// Size 20, Alignment 4, no padding. 
struct NozzleParams {
    speed_min: f32;
    speed_max: f32;
    angle_spread: f32;
    ttl_min: f32;
    ttl_max: f32;
};

// Size 16 + 32 + 20 = 68
// Alignment 8 -> padding = 72 - 68 = 4
struct EmitData {
    start_index: u32;
    num_emitted: u32;
    time: f32;
    dt: f32;

    motion: EmitterMotion;
    nozzle: NozzleParams;
    _padding: u32;
};

[[group(0), binding(0)]]
var<uniform> emit_data: EmitData;

struct Particles {
    data: [[stride(24)]] array<Particle>;
};
[[group(0), binding(1)]]
var<storage, read_write> particle_buffer: Particles;

// a % b
fn signed_mod(a: f32, b: f32) -> f32 {
  return a - floor(a/b) * b;
}

// Returns signed a - b
fn angle_difference(a: f32, b: f32) -> f32 {
  // https://stackoverflow.com/questions/1878907/the-smallest-difference-between-2-angles
  return signed_mod(a - b + PI / 2.0, PI) - PI / 2.0;
}

fn rotate2d(a: f32) -> mat2x2<f32> {
    let c = cos(a);
    let s = sin(a);
    return mat2x2<f32>(c,s,-s,c);
}

fn get_emit_index(global_id: u32, total_particles: u32) -> u32 {
    // "progress" in terms of number of emitted particles, the interval [0, num_emitted).
    var signed_emit_index = i32(global_id) - i32(emit_data.start_index);
    var emit_index: u32;
    if (signed_emit_index < 0) {
        // Wrap over the circular buffer.
        emit_index = u32(signed_emit_index + i32(total_particles));
    } else {
        emit_index = u32(signed_emit_index);
    }
    return emit_index;
}

[[stage(compute), workgroup_size(256)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>, [[builtin(num_workgroups)]] num_workgroups: vec3<u32>) {
    let total_particles = num_workgroups[0] * 256u;
    let gid = global_id[0];
    let particle = &(particle_buffer.data[gid]);

    let emit_index = get_emit_index(gid, total_particles);
    if (emit_index >= emit_data.num_emitted) {
        return;
    } 

    let num_passes_per_iteration: u32 = 5u;
    let emits_per_pass = u32(ceil(f32(emit_data.num_emitted) / f32(num_passes_per_iteration)));
    
    // The 'time' interpolation because we're approximating a continous stream with discrete time processing.
    let t_interp = f32(emit_index % emits_per_pass) / f32(emits_per_pass); 

    let local_emit_angle: f32 = 0.0;
    let unit_emit_rotation = vec2<f32>(cos(local_emit_angle), sin(local_emit_angle)); 
    let local_emit_speed = mix(emit_data.nozzle.speed_min, emit_data.nozzle.speed_max, .5);
    let local_emit_velocity = unit_emit_rotation * local_emit_speed;


    // Get the global frame of the ship. 
    let angle_delta = angle_difference(emit_data.motion.angle_end, emit_data.motion.angle_start); 
    let ship_angle = mix(emit_data.motion.angle_start + angle_delta, emit_data.motion.angle_start, t_interp);
    let ship_position = mix(emit_data.motion.position_end, emit_data.motion.position_start, t_interp); 

    let local_rotate_global = rotate2d(ship_angle);

    let center  = vec2<f32>(vec2<u32>(640u / 2u, 360u / 2u));

    (*particle).position = ship_position; 
    (*particle).velocity = local_rotate_global * local_emit_velocity + emit_data.motion.velocity;
    (*particle).ttl = 3.0;//mix(emit_data.nozzle.ttl_min, emit_data.nozzle.ttl_max, .5); 
}