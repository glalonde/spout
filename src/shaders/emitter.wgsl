{% include "particle.wgsl.include" %}
{% include "grid.wgsl.include" %}
{% include "hash.wgsl.include" %}
{% include "noise.wgsl.include" %}

let PI: f32 = 3.14159265358979323846;

// Size 40, Alignment 8, no padding.
// Pad out to multiple of 16 bytes.
struct EmitterMotion {
    position_start: vec2<f32>;
    position_end: vec2<f32>;
    velocity_start: vec2<f32>;
    velocity_end: vec2<f32>;
    angle_start: f32;
    angle_end: f32;
    _p0: u32;
    _p1: u32;
};

// Size 16, Alignment 4
struct NozzleParams {
    speed_min: f32;
    speed_max: f32;
    ttl_min: f32;
    ttl_max: f32;
};

// Size 16 + 40 + 4 + 32 = 84
// Alignment 8 -> No padding.
struct EmitData {
    start_index: u32;
    num_emitted: u32;
    time: f32;
    dt: f32;

    motion: EmitterMotion;
    nozzle: NozzleParams;
};

@group(0) @binding(0)
var<uniform> emit_data: EmitData;

@group(0) @binding(1)
var<storage, read_write> particle_buffer: array<Particle>;

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

// "progress" in terms of number of emitted particles, the interval [0, num_emitted).
// Distinct from global_id, which is index in the 'circular' buffer of particles.
fn get_emit_index(global_id: u32, total_particles: u32) -> u32 {
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

// The x shape of the wing
fn nozzle_shape(interp: f32) -> vec2<f32> {
  let rocket_width: f32 = 8.0;
  return vec2<f32>(0.0, mix(-rocket_width / 2.0, rocket_width / 2.0, interp));
}

@stage(compute) @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let total_particles = num_workgroups[0] * 256u;
    let gid = global_id[0];
    let particle = &(particle_buffer[gid]);

    let emit_index = get_emit_index(gid, total_particles);
    if (emit_index >= emit_data.num_emitted) {
        return;
    } 

    let emit_p = f32(emit_index) / f32(emit_data.num_emitted);
    let smooth_interp_time = emit_p * emit_data.dt + emit_data.time;

    // Rand number in interval [0, 1]
    let rand1 = hash11(10000.0*smooth_interp_time); 

    let emits_per_pass = 11u;

    let num_passes_per_iteration: u32 = u32(ceil(f32(emit_data.num_emitted) / f32(emits_per_pass)));
    
    // The 'time' interpolation because we're approximating a continous stream with discrete time processing.
    // let t_interp = f32(emit_index) / f32(emit_data.num_emitted); 

    let pass_index = emit_index / emits_per_pass; 
    let pass_t_interp = f32(pass_index) / f32(num_passes_per_iteration);
    let pass_time = pass_t_interp * emit_data.dt;

    // Local position interpolation to approximate emitting over a line rather than a point.
    let x_interp_step = 1.0 / f32(emits_per_pass - 1u);
    let x_interp = f32(emit_index % emits_per_pass) / f32(emits_per_pass) + x_interp_step * rand1;

    let interp_time = pass_time + emit_data.time;

    // Do all of the math as if the ship were at the origin oriented down the X axis, and then transform at the end.
    let tentacle_frequency = 25.0;
    let local_emit_angle = noise2d(vec2<f32>(x_interp * tentacle_frequency, interp_time)) - .5;
    let unit_emit_rotation = vec2<f32>(cos(local_emit_angle), sin(local_emit_angle)); 

    // let speed_noise_magnitude = 0.0;
    // let speed_noise = rand1 * speed_noise_magnitude; 
    let local_emit_speed = mix(emit_data.nozzle.speed_min, emit_data.nozzle.speed_max, 0.5);
    let local_emit_velocity = unit_emit_rotation * local_emit_speed;

    // The delta from iteration start to when this particle was emitted.
    let local_emit_position = nozzle_shape(x_interp);


    // Get the global frame of the ship. 
    let angle_delta = angle_difference(emit_data.motion.angle_end, emit_data.motion.angle_start); 
    let ship_angle = mix(emit_data.motion.angle_start, emit_data.motion.angle_start + angle_delta, pass_t_interp);
    let ship_position = mix(emit_data.motion.position_start, emit_data.motion.position_end, pass_t_interp); 
    let ship_velocity = mix(emit_data.motion.velocity_start, emit_data.motion.velocity_end, pass_t_interp); 

    let local_rotate_global = rotate2d(ship_angle);

    (*particle).position = ship_position + local_rotate_global * local_emit_position; 
    (*particle).velocity = local_rotate_global * local_emit_velocity;
    (*particle).ttl = mix(emit_data.nozzle.ttl_min, emit_data.nozzle.ttl_max, .5); 
    (*particle).local_dt = emit_data.dt - pass_time;
}