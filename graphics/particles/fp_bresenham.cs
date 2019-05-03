#version 430
#extension GL_ARB_compute_variable_group_size : enable
layout(location = 0) uniform float dt;
layout(location = 1) uniform int buffer_width;
layout(location = 2) uniform int buffer_height;
layout(location = 3) uniform float damage_rate;

layout(binding = 0, r32i) uniform iimage2D terrain_texture;
layout(binding = 1, r32ui) uniform uimage2D counter_texture;
layout(local_size_variable) in;

struct Particle {
  vec2 position;
  vec2 velocity;
};

layout(std430, binding = 0) buffer Particles {
  Particle particles[];
};

ivec2 GetCell(in vec2 pos) {
  return ivec2(floor(pos));
}

float Norm(in vec2 vel) {
  return sqrt(vel.x*vel.x + vel.y * vel.y);
}

bool OnBuffer(in ivec2 cell) {
  return cell.x >= 0 && cell.y >= 0 && cell.x < buffer_width &&
         cell.y < buffer_height;
}

// Returns true if bounce occurred.
bool TryErodeTerrain(in ivec2 cell, in float speed) {
  int test_value = imageLoad(terrain_texture, cell).x;
  if (test_value > 0) {
    int dmg_amt = int(damage_rate * speed);
    int actual_value = imageAtomicAdd(terrain_texture, cell, -dmg_amt);
    return actual_value > 0;
  }
  return false;
}

void main() {
  uint gid = gl_GlobalInvocationID.x;
  Particle p = particles[gid];
  vec2 signed_delta = p.velocity * dt;
  vec2 end_pos = p.position + signed_delta;

  vec2 delta = abs(signed_delta);
  ivec2 step =
      ivec2(signed_delta.x >= 0 ? 1 : -1, signed_delta.y >= 0 ? 1 : -1);
  ivec2 current_cell = GetCell(p.position);
  ivec2 end_cell = GetCell(end_pos);
  ivec2 delta_i = end_cell - current_cell;

  // Starting cell remainder:
  vec2 start_remainder = vec2(.5, .5) - (p.position - current_cell);
  start_remainder *= step;
  vec2 end_remainder = end_pos - end_cell;

  // Error value
  float error = delta.x * start_remainder.y - delta.y * start_remainder.x;

  // Update velocity
  vec2 vel_out = p.velocity;

  int num_cells = abs(delta_i.x) + abs(delta_i.y);
  while (num_cells > 0) {
    float error_horizontal = error - delta.y;
    float error_vertical = error + delta.x;
    if (error_vertical > -error_horizontal) {
      // Horizontal step
      error = error_horizontal;
      current_cell.x += step.x;
      // Check cell
      bool bounce;
      if (OnBuffer(current_cell)) {
        bounce = TryErodeTerrain(current_cell, Norm(vel_out));
      } else {
        bounce = true;
      }
      if (bounce) {
        // Bounce horizontally
        current_cell.x -= step.x;
        vel_out.x *= -1;
        end_remainder.y = 1.0 - end_remainder.y;
      }
    } else {
      // Vertical step
      error = error_vertical;
      current_cell.y += step.y;
      // Check cell
      bool bounce;
      if (OnBuffer(current_cell)) {
        bounce = TryErodeTerrain(current_cell, Norm(vel_out));
      } else {
        bounce = true;
      }
      if (bounce) {
        // Bounce vertically 
        current_cell.y -= step.y;
        vel_out.y *= -1;
        end_remainder.x = 1.0 - end_remainder.x;
      }
    }
    --num_cells;
  }

  particles[gid].position = vec2(current_cell) + end_remainder;
  particles[gid].velocity = vel_out;

  // Draw to the density texture
  imageAtomicAdd(counter_texture, current_cell, 1);
}
