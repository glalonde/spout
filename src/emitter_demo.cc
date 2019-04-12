#include <thread>
#include <random>

#include "base/format.h"
#include "base/init.h"
#include "graphics/animated_canvas.h"
#include "src/bresenham.h"
#include "src/convert.h"
#include "src/demo_utils.h"
#include "src/emitter.h"
#include "src/fonts/font_renderer.h"
#include "src/random.h"

DEFINE_int32(emission_rate, 100, "Number of particles per second");

void RenderParticle(const Vector2d& pos, Image<PixelType::RGBAU8>* data) {
  // (x, y) -> (col, height - row)
  Vector2i pos_i = pos.cast<int>();
  pos_i[1] = pos_i[1];
  (*data)(pos_i[1], pos_i[0]) = kParticleColor;
}

void UpdateParticles(const double dt, const double ddy,
                     CircularBuffer<Vector5d>* particles,
                     Image<uint8_t>* environment,
                     Image<PixelType::RGBAU8>* data) {
  const int num_particles = particles->Capacity();
  auto& mutable_particles = *particles->mutable_data();
  for (int i = 0; i < num_particles; ++i) {
    auto& current = mutable_particles[i]; 
    if (current[4] /* ttl */ <= 0) {
      continue;
    }
    Vector2d pos;
    Vector2d vel;
    DestructingBresenham(current.segment<2>(0), current.segment<2>(2), dt, 1.0,
                         environment, &pos, &vel);
    current.segment<2>(0) = pos;
    current.segment<2>(2) = vel;
    current[3] += dt * ddy;
    current[4] -= dt;
    RenderParticle(current.segment<2>(0), data);
  }
}

void Demo(double emission_rate) {
  // Set up canvas
  const double kFps = 60.0;
  const Vector2i window_dims(800, 800);
  const Vector2i grid_dims = window_dims / 4;
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], kFps);

  std::mt19937 rand_gen(0);

  // Set up environment
  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  AddNoise(kWall, .2, &rand_gen, &environment);
  AddAllWalls(kWall, &environment);

  // Make emitter
  const double angular_stdev = .2;
  const double min_speed = 30.0;
  const double max_speed = 70.0;
  const double particle_life = 3;
  Emitter e(angular_stdev, min_speed, max_speed, emission_rate, particle_life);

  const SO2d start_angle(0.0);
  const SO2d end_angle(0.0);
  const Vector2d start_pos(100.0, 100.0);
  const Vector2d end_pos(100.0, 100.0);
  const Vector2d start_vel(0, 0);
  const Vector2d end_vel(0, 0);


  const double ddy = -9.81;
  auto* data = canvas.data();
  Duration dt(0);
  ControllerInput input;
  while (!input.quit) {
    const double n_seconds = ToSeconds<double>(dt);
    if (input.up) {
      e.EmitOverTime(n_seconds, start_angle, end_angle, start_pos, end_pos,
                     start_vel, end_vel);
    }
    RenderEnvironment(environment, data);
    UpdateParticles(n_seconds, ddy, e.mutable_particles(), &environment, data);
    AddFpsText(canvas.fps(), text_color, data);
    input = canvas.Tick(&dt);
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Demo(FLAGS_emission_rate);
  return 0;
}
