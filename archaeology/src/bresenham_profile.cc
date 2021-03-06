#include <random>

#include "base/format.h"
#include "base/init.h"
#include "base/scoped_profiler.h"
#include "base/wall_timer.h"
#include "src/bresenham.h"
#include "src/convert.h"
#include "src/demo_utils.h"
#include "src/random.h"

ABSL_FLAG(int32_t, num_particles, 100, "Number of particles");
ABSL_FLAG(bool, floating_point, false, "Use floating point?");
ABSL_FLAG(bool, low_res, false, "Use floating point?");

void DemoLowResInteger(int num_particles) {
  const double kFps = 60.0;
  const int cell_size = kCellSize<uint32_t, 8>;
  const Vector2i window_dims(800, 800);
  const Vector2i grid_dims = window_dims / 4;

  std::mt19937 rand_gen(0);

  // Set up environment
  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  AddAllWalls(kWall, &environment);

  // Set up particles
  auto dist = UniformRandomDistribution<int>(-30 * cell_size, 30 * cell_size);
  std::vector<std::pair<Vector2u32, Vector2i>> particles(num_particles);
  for (int i = 0; i < num_particles; ++i) {
    particles[i].first =
        Vector2u32::Constant(SetLowRes<8>(kAnchor<uint32_t, 8>));
    particles[i].first += Vector2u32::Constant(50 * cell_size);
    particles[i].second = Vector2i(dist(rand_gen), dist(rand_gen));
  }

  Vector2u32 pos;
  Vector2i vel;
  double dt = ToSeconds<double>(FromHz(kFps));
  const double ddy = -9.81 * cell_size;
  WallTimer timer;
  timer.Start();
  ScopedProfiler prof;
  int num_iters = 0;
  while (timer.ElapsedDuration() < FromSeconds<double>(10.0)) {
    for (int i = 0; i < num_particles; ++i) {
      BresenhamExperimentLowRes(particles[i].first, particles[i].second, dt,
                                environment, &pos, &vel);
      particles[i].first = pos;
      particles[i].second = vel;
      particles[i].second[1] += static_cast<int>(dt * ddy);
    }
    ++num_iters;
  }
  LOG(INFO) << "Completed " << num_iters << " iterations.";
}

void DemoInteger(int num_particles) {
  const double kFps = 60.0;
  const int kCellSize = 500;
  const Vector2i window_dims(800, 800);
  const Vector2i grid_dims = window_dims / 4;

  std::mt19937 rand_gen(0);

  // Set up environment
  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  AddAllWalls(kWall, &environment);

  // Set up particles
  auto dist = UniformRandomDistribution<int>(-30 * kCellSize, 30 * kCellSize);
  std::vector<Vector4i> particles(num_particles);
  for (int i = 0; i < num_particles; ++i) {
    particles[i] = Vector4i(50 * kCellSize, 50 * kCellSize, dist(rand_gen),
                            dist(rand_gen));
  }

  Vector2i pos;
  Vector2i vel;
  double dt = ToSeconds<double>(FromHz(kFps));
  const double ddy = -9.81 * kCellSize;
  WallTimer timer;
  timer.Start();
  ScopedProfiler prof;
  int num_iters = 0;
  while (timer.ElapsedDuration() < FromSeconds<double>(10.0)) {
    for (int i = 0; i < num_particles; ++i) {
      BresenhamExperiment(particles[i].segment<2>(0),
                          particles[i].segment<2>(2), kCellSize, dt,
                          environment, &pos, &vel);
      particles[i].segment<2>(0) = pos;
      particles[i].segment<2>(2) = vel;
      particles[i][3] += static_cast<int>(dt * ddy);
    }
    ++num_iters;
  }
  LOG(INFO) << "Completed " << num_iters << " iterations.";
}

void DemoFloatingPoint(int num_particles) {
  const double kFps = 60.0;
  const Vector2i window_dims(800, 800);
  const Vector2i grid_dims = window_dims / 4;

  std::mt19937 rand_gen(0);

  // Set up environment
  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  AddAllWalls(kWall, &environment);

  // Set up particles
  auto dist = UniformRandomDistribution<double>(-30.0, 30.0);
  std::vector<Vector4d> particles(num_particles);
  for (int i = 0; i < num_particles; ++i) {
    particles[i] = Vector4d(50, 50, dist(rand_gen), dist(rand_gen));
  }

  Vector2d pos;
  Vector2d vel;
  double dt = ToSeconds<double>(FromHz(kFps));
  const double ddy = -9.81;
  WallTimer timer;
  timer.Start();
  ScopedProfiler prof;
  int num_iters = 0;
  while (timer.ElapsedDuration() < FromSeconds<double>(10.0)) {
    for (int i = 0; i < num_particles; ++i) {
      SubPixelBresenhamNormal(particles[i].segment<2>(0),
                              particles[i].segment<2>(2), dt, environment, &pos,
                              &vel);
      particles[i].segment<2>(0) = pos;
      particles[i].segment<2>(2) = vel;
      particles[i][3] += dt * ddy;
    }
    ++num_iters;
  }
  LOG(INFO) << "Completed " << num_iters << " iterations.";
}

int main(int argc, char** argv) {
  Init(argc, argv);
  if (absl::GetFlag(FLAGS_floating_point)) {
    DemoFloatingPoint(absl::GetFlag(FLAGS_num_particles));
  } else if (absl::GetFlag(FLAGS_low_res)) {
    DemoLowResInteger(absl::GetFlag(FLAGS_num_particles));
  } else {
    DemoInteger(absl::GetFlag(FLAGS_num_particles));
  }
  return 0;
}
