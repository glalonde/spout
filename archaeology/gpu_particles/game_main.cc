#include "base/init.h"
#include "gpu_particles/game_controller.h"
#include "src/fps_estimator.h"

ABSL_FLAG(int32_t, color_map_index, 0, "Color map index, see color_maps.h");
ABSL_FLAG(double, damage_rate, 1.0, "Damage rate");
ABSL_FLAG(double, dt, .016, "Simulation rate");
ABSL_FLAG(int32_t, pixel_size, 4, "Pixel size");
ABSL_FLAG(double, emission_speed_min, 25.0, "Particle speed");
ABSL_FLAG(double, emission_speed_max, 75.0, "Particle speed");
ABSL_FLAG(double, emission_rate, 250.0, "Particle emission rate");
ABSL_FLAG(double, min_life, 1.0, "Min particle life");
ABSL_FLAG(double, max_life, 5.0, "Max particle life");

GameParameters ParseParametersFromFlags(int window_width, int window_height) {
  GameParameters params;
  params.grid_width = window_width / absl::GetFlag(FLAGS_pixel_size);
  params.grid_height = window_height / absl::GetFlag(FLAGS_pixel_size);
  params.damage_rate = absl::GetFlag(FLAGS_damage_rate);
  params.dt = absl::GetFlag(FLAGS_dt);
  params.particle_color_map_index = absl::GetFlag(FLAGS_color_map_index);
  params.emitter_params.emission_rate = absl::GetFlag(FLAGS_emission_rate);
  params.emitter_params.emission_speed_min =
      absl::GetFlag(FLAGS_emission_speed_min);
  params.emitter_params.emission_speed_max =
      absl::GetFlag(FLAGS_emission_speed_max);
  params.emitter_params.min_particle_life = absl::GetFlag(FLAGS_min_life);
  params.emitter_params.max_particle_life = absl::GetFlag(FLAGS_max_life);
  return params;
}

void TestLoop() {
  int window_width = 1440;
  int window_height = 900;
  GameParameters params = ParseParametersFromFlags(window_width, window_height);
  params.mantissa_bits = 12;
  params.emitter_params.cell_size = CellSize<uint32_t>(params.mantissa_bits);
  ParticleSim sim(window_width, window_height, params);
  FPSEstimator fps(FromSeconds(1), 60.0);
  double dt = params.dt;
  ControllerInput input;
  while (!input.quit) {
    input = sim.Update(dt);
    fps.Tick();
    LOG(INFO) << fps.CurrentEstimate();
  }
}

int main(int argc, char* argv[]) {
  Init(argc, argv);
  TestLoop();
  return 0;
}
