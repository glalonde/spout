#include <benchmark/benchmark.h>
#include "base/logging.h"
#include "src/random.h"

int floor_div(int a, int b) {
  int d = a / b;
  int r = a % b; /* optimizes into single division. */
  return r ? (d - ((a < 0) ^ (b < 0))) : d;
}

Vector2i GetCell1(const Vector2i& vec, int cell_size) {
  return vec.unaryExpr([&cell_size](int v) { return floor_div(v, cell_size); });
}

Vector2i GetCell2(const Vector2i& vec, int cell_size) {
  return Vector2i(floor_div(vec.x(), cell_size), floor_div(vec.y(), cell_size));
}

Vector2i GetCell3(const Vector2i& vec, int cell_size) {
  return (vec.cast<double>() / cell_size).array().floor().matrix().cast<int>();
}

Vector2i GetCell4(const Vector2i& vec, int cell_size) {
  return vec / cell_size;
}

std::vector<Vector2i> GetPoints(int n) {
  std::mt19937 gen(0);
  auto dist = UniformRandomDistribution<int>(-100, 100);
  std::vector<Vector2i> out(n);
  for (int i = 0; i < n; ++i) {
    out[i] = Vector2i(dist(gen), dist(gen));
  }
  return out;
}

static void BM_GetCell1(benchmark::State& state) {
  int n_points = 100000;
  auto points = GetPoints(n_points);
  std::vector<Vector2i> out_points(n_points);
  for (auto _ : state) {
    for (int i = 0; i < n_points; ++i) {
      out_points[i] = GetCell1(points[i], 10);
    }
  }
}

static void BM_GetCell2(benchmark::State& state) {
  int n_points = 100000;
  auto points = GetPoints(n_points);
  std::vector<Vector2i> out_points(n_points);
  for (auto _ : state) {
    for (int i = 0; i < n_points; ++i) {
      out_points[i] = GetCell2(points[i], 10);
    }
  }
}

static void BM_GetCell3(benchmark::State& state) {
  int n_points = 100000;
  auto points = GetPoints(n_points);
  std::vector<Vector2i> out_points(n_points);
  for (auto _ : state) {
    for (int i = 0; i < n_points; ++i) {
      out_points[i] = GetCell3(points[i], 10);
    }
  }
}

// Not correct
static void BM_GetCell4(benchmark::State& state) {
  int n_points = 100000;
  auto points = GetPoints(n_points);
  std::vector<Vector2i> out_points(n_points);
  for (auto _ : state) {
    for (int i = 0; i < n_points; ++i) {
      out_points[i] = GetCell4(points[i], 10);
    }
  }
}

BENCHMARK(BM_GetCell1);
BENCHMARK(BM_GetCell2);
BENCHMARK(BM_GetCell3);
BENCHMARK(BM_GetCell4);

BENCHMARK_MAIN();
