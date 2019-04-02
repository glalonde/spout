#include <thread>

#include "base/init.h"
#include "base/time.h"
#include "new_spout/bresenham.h"
#include "new_spout/controller_input.h"
#include "new_spout/particle.h"
#include "new_spout/sdl_container.h"

void Demo(int num_particles) {
  const Vector2i grid_dims(100, 100);
  const Vector2i screen_dims(1000, 1000);
  PixelBuffer<uint32_t> buffer(grid_dims);
  SDLContainer sdl(screen_dims, grid_dims);
  ControllerInput input;
  Eigen::AlignedBox<double, 5> particle_space;
  {
    particle_space.max() << 50, 50, 30, 30, 10;
    particle_space.min() << 50, 50, -30, -30, 10;
  }
  std::vector<Particle<double>> particles(num_particles);
  for (int i = 0; i < num_particles; ++i) {
    particles[i] = particle_space.sample();
  }

  buffer.Clear();
  for (int r = 0; r < grid_dims.y(); ++r) {
    buffer(r, 0) = kWall;
    buffer(r, grid_dims.x() - 1) = kWall;
  }
  for (int c = 0; c < grid_dims.x(); ++c) {
    buffer(0, c) = kWall;
    buffer(grid_dims.y() - 1, c) = kWall;
  }

  Vector2d pos;
  Vector2d vel;
  double dt = 1.0 / 60.0;
  const double ddy = -9.81;
  while (!input.quit) {
    auto start = ClockType::now();
    for (int i = 0; i < num_particles; ++i) {
      SubPixelBresenhamNormal(particles[i].segment<2>(0),
                              particles[i].segment<2>(2), dt, &buffer, &pos,
                              &vel);
      particles[i].segment<2>(0) = pos;
      particles[i].segment<2>(2) = vel;
      particles[i][3] += dt * ddy;
    }

    sdl.UpdateInput(&input);
    sdl.Render(buffer);
    std::this_thread::sleep_until(start + FromSeconds<double>(dt));
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Demo(100);
  return 0;
}
