#pragma once

struct EmitterParameters {
  int cell_size;
  float emission_rate;
  float emission_speed_min;
  float emission_speed_max;
  float min_particle_life;
  float max_particle_life;
};

struct GameParameters {
  int grid_width;
  int grid_height;
  int mantissa_bits;
  double damage_rate;
  double dt;
  EmitterParameters emitter_params;
  int particle_color_map_index;
};
