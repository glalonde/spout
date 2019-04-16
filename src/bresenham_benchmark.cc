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
