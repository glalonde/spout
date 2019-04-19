#include <benchmark/benchmark.h>
#include "base/format.h"
#include "base/logging.h"
#include "src/bresenham.h"
#include "src/demo_utils.h"
#include "src/random.h"
#include "src/so2.h"

namespace {
Image<uint8_t> MakeEmptyEnvironment(int rows, int cols) {
  Image<uint8_t> environment(rows, cols);
  environment.setConstant(0);
  AddAllWalls(kWall, &environment);
  return environment;
}
}  // namespace

static void BM_Bresenham(benchmark::State& state) {
  Vector2d pos;
  Vector2d vel;
  double dt;
  std::mt19937 gen(0);
  static auto env = MakeEmptyEnvironment(480, 640);
  auto x_dist = UniformRandomDistribution(1.5, 638.5);
  auto y_dist = UniformRandomDistribution(1.5, 478.5);
  auto angle_dist = UniformRandomDistribution(-M_PI , M_PI);

  // Choose random vectors (position and velocity) in the the workspace and
  // rasterize a line with bouncing.
  auto set_random_problem = [&](const double velocity, const double fps) {
    pos.x() = x_dist(gen);
    pos.y() = y_dist(gen);
    SO2d angle(angle_dist(gen));
    vel = angle.data() * velocity;
    dt = 1.0 / fps;
  };
  for (auto _ : state) {
    state.PauseTiming();
    set_random_problem(state.range(0), state.range(1));
    state.ResumeTiming();
    SubPixelBresenhamNormal(pos, vel, dt, env, &pos, &vel);
  }
}

BENCHMARK(BM_Bresenham)->Ranges({{250, 1000 /* velocity */}, {15, 60 /* fps */}});

BENCHMARK_MAIN();

/*
 *
 *
 *
 * BASELINE:
 Run on (20 X 4500 MHz CPU s)
CPU Caches:
  L1 Data 32K (x10)
  L1 Instruction 32K (x10)
  L2 Unified 1024K (x10)
  L3 Unified 14080K (x1)
Load Average: 1.21, 0.98, 0.72
---------------------------------------------------------------
Benchmark                     Time             CPU   Iterations
---------------------------------------------------------------
BM_Bresenham/250/15         441 ns          442 ns      1591808
BM_Bresenham/512/15         484 ns          485 ns      1444299
BM_Bresenham/1000/15        554 ns          555 ns      1263196
BM_Bresenham/250/60         399 ns          400 ns      1750862
BM_Bresenham/512/60         417 ns          418 ns      1692747
BM_Bresenham/1000/60        439 ns          440 ns      1590648*
*/
