#include <random>
#include <thread>

#include "base/format.h"
#include "base/init.h"
#include "src/convert.h"
#include "src/image_viewer/animated_canvas.h"
#include "src/level_generators.h"

DEFINE_int32(level_number, 1, "Level number");

static constexpr uint8_t kWall = std::numeric_limits<uint8_t>::max();
static const PixelType::RGBAU8 kWallColor = {0, 0, 255, 255};

void RenderEnvironment(const Image<uint8_t>& env,
                       Image<PixelType::RGBAU8>* data) {
  for (int r = 0; r < env.rows(); ++r) {
    for (int c = 0; c < env.cols(); ++c) {
      const int ar = env.rows() - r - 1;
      if (env(r, c) > 0) {
        (*data)(ar, c) = kWallColor;
      } else {
        (*data)(ar, c) = {0, 0, 0, 255};
      }
    }
  }
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

void Demo(int level_number) {
  // Set up canvas
  const double kFps = 60.0;
  const Vector2i window_dims(800, 800);
  const Vector2i grid_dims = window_dims / 4;
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], kFps);

  // Set up environment
  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  constexpr uint8_t kMaxObs = std::numeric_limits<uint8_t>::max();
  constexpr uint8_t kMinObs = kMaxObs / 2;
  MakeLevel(kMinObs, kMaxObs, level_number, 0, &environment);
  AddWalls(kWall, &environment);

  // Loop
  bool done = false;
  auto* data = canvas.data();
  while (!done) {
    RenderEnvironment(environment, data);
    done = canvas.Tick().quit;
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Demo(FLAGS_level_number);
  return 0;
}