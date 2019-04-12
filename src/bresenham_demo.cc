#include <thread>
#include <random>

#include "base/format.h"
#include "base/init.h"
#include "src/bresenham.h"
#include "src/convert.h"
#include "src/fonts/font_renderer.h"
#include "graphics/animated_canvas.h"
#include "src/random.h"
#include "src/demo_utils.h"

DEFINE_int32(num_particles, 100, "Number of particles");

void RenderParticle(const Vector2d& pos, Image<PixelType::RGBAU8>* data) {
  // (x, y) -> (col, height - row)
  Vector2i pos_i = pos.cast<int>();
  pos_i[1] = pos_i[1];
  (*data)(pos_i[1], pos_i[0]) = kParticleColor;
}

void Demo(int num_particles) {
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

  // Set up particles
  AlignedBox<double, 4> particle_space;
  {
    particle_space.max() << 50, 50, 30, 30;
    particle_space.min() << 50, 50, -30, -30;
  }
  std::vector<Vector4d> particles(num_particles);
  for (int i = 0; i < num_particles; ++i) {
    particles[i] = particle_space.sample();
  }

  Vector2d pos;
  Vector2d vel;
  double dt = ToSeconds<double>(FromHz(kFps));
  const double ddy = -9.81;

  bool done = false;
  auto* data = canvas.data();
  while (!done) {
    RenderEnvironment(environment, data);
    for (int i = 0; i < num_particles; ++i) {
      DestructingBresenham(particles[i].segment<2>(0),
                           particles[i].segment<2>(2), dt, 1.0, &environment,
                           &pos, &vel);
      particles[i].segment<2>(0) = pos;
      particles[i].segment<2>(2) = vel;
      particles[i][3] += dt * ddy;
      RenderParticle(pos, data);
    }
    AddFpsText(canvas.fps(), text_color, data);
    done = canvas.Tick().quit;
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Demo(FLAGS_num_particles);
  return 0;
}
