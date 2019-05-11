#include "base/init.h"
#include "gpu_particles/game_controller.h"
#include "src/fps_estimator.h"

DEFINE_int32(color_map_index, 0, "Color map index, see color_maps.h");
DEFINE_double(damage_rate, 1.0, "Damage rate");
DEFINE_double(dt, .016, "Simulation rate");
DEFINE_int32(pixel_size, 4, "Pixel size");
DEFINE_double(emission_speed_min, 75.0, "Particle speed");
DEFINE_double(emission_speed_max, 150.0, "Particle speed");
DEFINE_double(emission_rate, 250.0, "Particle emission rate");
DEFINE_double(min_life, 1.0, "Min particle life");
DEFINE_double(max_life, 5.0, "Max particle life");

GameParameters ParseParametersFromFlags(int window_width, int window_height) {
  GameParameters params;
  params.grid_width = window_width / FLAGS_pixel_size;
  params.grid_height = window_height / FLAGS_pixel_size;
  params.damage_rate = FLAGS_damage_rate;
  params.dt = FLAGS_dt;
  params.particle_color_map_index = FLAGS_color_map_index;
  params.emitter_params.emission_rate = FLAGS_emission_rate;
  params.emitter_params.emission_speed_min = FLAGS_emission_speed_min;
  params.emitter_params.emission_speed_max = FLAGS_emission_speed_max;
  params.emitter_params.min_particle_life = FLAGS_min_life;
  params.emitter_params.max_particle_life = FLAGS_max_life;
  return params;
}

void TestLoop() {
  int window_width = 1440;
  int window_height = 900;
  GameParameters params = ParseParametersFromFlags(window_width, window_height);
  params.mantissa_bits = 14;
  params.emitter_params.cell_size = CellSize<uint32_t>(params.mantissa_bits);
  ParticleSim sim(window_width, window_height, params);
  FPSEstimator fps(FromSeconds(2), 60.0);
  double dt = params.dt;
  ControllerInput input;
  TimePoint previous_cycle = ClockType::now();
  while (!input.quit) {
    input = sim.Update(dt);
    const auto now = ClockType::now();
    const auto delta = now - previous_cycle;
    previous_cycle = now;
    double estimated_fps = fps.Tick(delta);
    LOG(INFO) << estimated_fps;
  }
}

int main(int argc, char* argv[]) {
  Init(argc, argv);
  TestLoop();
  return 0;
}
