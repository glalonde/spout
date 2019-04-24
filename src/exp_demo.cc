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

DEFINE_bool(run_demo, false, "run demo?");

void RenderParticle(const Vector2i& pos, const int cell_size,
                    Image<PixelType::RGBAU8>* data) {
  // (x, y) -> (col, height - row)
  Vector2i pos_i = pos / cell_size;
  (*data)(pos_i[1], pos_i[0]) = kParticleColor;
}

void Demo() {
  // Set up canvas
  const double kFps = 60.0;
  const Vector2i window_dims(640, 480);
  const Vector2i grid_dims = window_dims / 4;
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], kFps);

  std::mt19937 rand_gen(0);

  const int kCellSize = 100;
  // Set up environment
  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  AddAllWalls(kWall, &environment);

  Vector4i particle = Vector4i::Zero();
  particle.head<2>() = grid_dims / 2 * kCellSize;
  particle.tail<2>().x() = 60.0 * kCellSize;
  particle.tail<2>().y() = 32.0 * kCellSize;

  Vector2i pos;
  Vector2i vel;
  double dt = ToSeconds<double>(FromHz(kFps));
  const double ddy = -9.81;

  bool done = false;
  auto* data = canvas.data();
  while (!done) {
    RenderEnvironment(environment, data);
    BresenhamExperiment(particle.segment<2>(0), particle.segment<2>(2),
                        kCellSize, dt, environment, &pos, &vel);
    particle.segment<2>(0) = pos;
    particle.segment<2>(2) = vel;
    // particle[3] += (dt * ddy * kCellSize);
    RenderParticle(pos, kCellSize, data);
    AddFpsText(canvas.fps(), text_color, data);
    done = canvas.Tick().quit;
  }
}

void Test() {
  const int kCellSize = 100;
  // Set up environment
  Image<uint8_t> environment(100, 200);
  environment.setConstant(0);

  Vector4i particle = Vector4i::Zero();
  particle.head<2>() =
      Vector2i(0, 50) * kCellSize + Vector2i(kCellSize / 2, kCellSize / 2);

  particle.tail<2>().x() = .06 * kCellSize;
  particle.tail<2>().y() = -.16 * kCellSize;

  auto step = [&]() {
    LOG(INFO) << "STEPPPPPPPP";
    LOG(INFO) << "Start: " << particle.transpose();
    Vector2i pos;
    Vector2i vel;
    BresenhamExperiment(particle.segment<2>(0), particle.segment<2>(2),
                        kCellSize, 1.0, environment, &pos, &vel);
    particle.segment<2>(0) = pos;
    particle.segment<2>(2) = vel;
    LOG(INFO) << "End: " << particle.transpose();
  };
  step();
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Test();
  if (FLAGS_run_demo) {
    Demo();
  }
  return 0;
}
