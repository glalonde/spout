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

std::vector<std::pair<Vector2d, Vector2d>> GetProblems(int n_problems,
                                                       double velocity) {
  std::mt19937 gen(0);
  static auto env = MakeEmptyEnvironment(480, 640);
  auto x_dist = UniformRandomDistribution(1.5, 638.5);
  auto y_dist = UniformRandomDistribution(1.5, 478.5);
  auto angle_dist = UniformRandomDistribution(-M_PI , M_PI);
  std::vector<std::pair<Vector2d, Vector2d>> out(n_problems);
  for (int i = 0; i < n_problems; ++i) {
    auto& pos = out[i].first;
    auto& vel = out[i].second;
    pos.x() = x_dist(gen);
    pos.y() = y_dist(gen);
    SO2d angle(angle_dist(gen));
    vel = angle.data() * velocity;
  }
  return out;
}

std::vector<std::pair<Vector2i, Vector2i>> GetIntegerProblems(int n_problems,
                                                              double velocity,
                                                              int cell_size) {
  auto fp_probs = GetProblems(n_problems, velocity);
  std::vector<std::pair<Vector2i, Vector2i>> out(n_problems);
  for (int i = 0; i < n_problems; ++i) {
    out[i].first = (fp_probs[i].first * cell_size).cast<int>();
    out[i].second = (fp_probs[i].second * cell_size).cast<int>();
  }
  return out;
}
}  // namespace

static void BM_Bresenham(benchmark::State& state) {
  static auto env = MakeEmptyEnvironment(480, 640);
  const double dt = 1.0 / state.range(1);
  auto problems = GetProblems(1000, state.range(0));
  Vector2d pos;
  Vector2d vel;
  for (auto _ : state) {
    for (const auto& p : problems) {
      SubPixelBresenhamNormal(p.first, p.second, dt, env, &pos, &vel);
    }
  }
}

static void BM_BresenhamInteger(benchmark::State& state) {
  static auto env = MakeEmptyEnvironment(480, 640);
  const int kCellSize = 100;
  const double dt = 1.0 / state.range(1);
  auto problems = GetIntegerProblems(1000, state.range(0), kCellSize);
  Vector2i pos;
  Vector2i vel;
  for (auto _ : state) {
    for (const auto& p : problems) {
      BresenhamExperiment(p.first, p.second, kCellSize, dt, env, &pos, &vel);
    }
  }
}

BENCHMARK(BM_Bresenham)
    ->Ranges({{10, 1000 /* velocity */}, {15, 60 /* fps */}});
BENCHMARK(BM_BresenhamInteger)
    ->Ranges({{10, 1000 /* velocity */}, {15, 60 /* fps */}});

BENCHMARK_MAIN();
