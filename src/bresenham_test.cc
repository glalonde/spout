#include <thread>

#include "base/googletest.h"
#include "base/time.h"
#include "src/bresenham.h"
#include "src/image_viewer/animated_canvas.h"

static constexpr uint8_t kWall = std::numeric_limits<uint8_t>::max();
static const PixelType::RGBAU8 kWallColor = {0, 0, 255, 255};
static const PixelType::RGBAU8 kParticleColor = {0, 255, 0, 255};

void RenderEnvironment(const Image<uint8_t>& env,
                       Image<PixelType::RGBAU8>* data) {
  for (int c = 0; c < env.cols(); ++c) {
    for (int r = 0; r < env.rows(); ++r) {
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

GTEST_TEST(BresenhamTest, SmokeVis) {
  // Width, height (not rows, cols)
  const Vector2i window_dims(400, 400);
  const Vector2i grid_dims = window_dims / 4;
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], 60.0);

  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  const uint8_t kWall = std::numeric_limits<uint8_t>::max();
  // Set left and right to walls
  for (int r = 0; r < grid_dims.y(); ++r) {
    environment(r, 0) = kWall;
    environment(r, grid_dims.x() - 1) = kWall;
  }
  // Set top and bottom to walls
  for (int c = 0; c < grid_dims.x(); ++c) {
    environment(0, c) = kWall;
    environment(grid_dims.y() - 1, c) = kWall;
  }

  // Create 1 particle
  Vector2d pos(50, 20);
  Vector2d vel = Vector2d(7, 6.9).normalized() * 150;
  double dt = 1.0 / 300.0;

  bool done = false;
  auto* data = canvas.data();
  while (!done) {
    SubPixelBresenhamNormal(pos, vel, dt, &environment, &pos, &vel);
    RenderEnvironment(environment, data);
    RenderParticle(pos, data);
    done = canvas.Tick().quit;
  }
}

GTEST_MAIN();
