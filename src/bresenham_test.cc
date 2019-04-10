#include <thread>

#include "base/googletest.h"
#include "src/bresenham.h"
#include "graphics/animated_canvas.h"

static constexpr uint8_t kWall = std::numeric_limits<uint8_t>::max();
static const PixelType::RGBAU8 kWallColor = {0, 0, 255, 255};
static const PixelType::RGBAU8 kParticleColor = {0, 255, 0, 255};
static const PixelType::RGBAU8 kTrailColor = {0, 128, 0, 255};

void RenderEnvironment(const Image<uint8_t>& env,
                       Image<PixelType::RGBAU8>* data) {
  for (int r = 0; r < env.rows(); ++r) {
    for (int c = 0; c < env.cols(); ++c) {
      if (env(r, c) == kWall) {
        (*data)(r, c) = kWallColor;
      } else {
        (*data)(r, c) = {0, 0, 0, 255};
      }
    }
  }
}

void RenderParticle(const Vector2d& pos, Image<PixelType::RGBAU8>* data) {
  // (x, y) -> (col, height - row)
  Vector2i pos_i = pos.cast<int>();
  pos_i[1] = data->rows() - pos_i[1] - 1;
  (*data)(pos_i[1], pos_i[0]) = kParticleColor;
}

template <class T>
void AddWalls(const T& wall_value, Image<T>* data) {
  // Set left and right to walls
  for (int r = 0; r < data->rows(); ++r) {
    (*data)(r, 0) = wall_value;
    (*data)(r, data->cols() - 1) = wall_value;
  }
  // Set top and bottom to walls
  for (int c = 0; c < data->cols(); ++c) {
    (*data)(0, c) = wall_value;
    (*data)(data->rows() - 1, c) = wall_value;
  }
}

GTEST_TEST(BresenhamTest, SmokeVis) {
  const double kFps = 60.0;
  // Width, height (not rows, cols)
  const Vector2i window_dims(400, 400);
  const Vector2i grid_dims = window_dims / 4;
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], kFps);

  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  AddWalls(kWall, &environment);

  // Create 1 particle
  Vector2d pos(50, 20);
  Vector2d vel = Vector2d(7, 6.9).normalized() * 10.0;
  double dt = 1.0 / kFps;
  std::vector<Vector2i> trails;

  bool done = false;
  auto* data = canvas.data();
  while (!done && !kIsUnitTest) {
    SubPixelBresenhamNormal(pos, vel, dt, environment, &pos, &vel);
    RenderEnvironment(environment, data);
    RenderParticle(pos, data);
    done = canvas.Tick().quit;
  }
}

GTEST_MAIN();
