#pragma once

struct EmitterParameters {
  float emission_rate;
  float emission_speed_min;
  float emission_speed_max;
  float min_particle_life;
  float max_particle_life;
};

struct GameParameters {
  int grid_width;
  int grid_height;
  double damage_rate;
  double dt;
  EmitterParameters emitter_params;
  int particle_color_map_index;
};
