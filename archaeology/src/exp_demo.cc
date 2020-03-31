#include <thread>
#include <random>

#include "base/format.h"
#include "base/init.h"
#include "graphics/animated_canvas.h"
#include "src/bresenham.h"
#include "src/convert.h"
#include "src/demo_utils.h"
#include "src/fonts/font_renderer.h"
#include "src/int_grid.h"
#include "src/random.h"

ABSL_FLAG(bool, run_demo, false, "run demo?");

void RenderParticle(const Vector2u32& pos, Image<PixelType::RGBAU8>* data) {
  auto get_cell = [](const Vector2u32& vec) -> Vector2i {
    return vec.unaryExpr([](uint32_t v) -> int {
      return static_cast<int>(GetLowRes<8>(v)) - kAnchor<uint32_t, 8>;
    });
  };
  // (x, y) -> (col, height - row)
  Vector2i pos_i = get_cell(pos);
  LOG(INFO) << pos_i.transpose();
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

  const int cell_size = kCellSize<uint32_t, 8>;
  // Set up environment
  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  AddAllWalls(kWall, &environment);

  Vector2u32 pos;
  pos.x() =
      SetLowRes<8>(kAnchor<uint32_t, 8>) + (grid_dims.x() / 2 * cell_size);
  pos.y() =
      SetLowRes<8>(kAnchor<uint32_t, 8>) + (grid_dims.y() / 2 * cell_size);
  LOG(INFO) << "Starting position: " << pos.transpose();
  Vector2i vel = Vector2i(-32, -60) * cell_size;

  Vector2u32 pos_next;
  Vector2i vel_next;
  double dt = ToSeconds<double>(FromHz(kFps));
  const double ddy = -9.81;

  bool done = false;
  auto* data = canvas.data();
  while (!done) {
    RenderEnvironment(environment, data);
    BresenhamExperimentLowRes(pos, vel, dt, environment, &pos_next, &vel_next);
    pos = pos_next;
    vel = vel_next;
    // particle[3] += (dt * ddy * kCellSize);
    RenderParticle(pos, data);
    AddFpsText(canvas.fps(), text_color, data);
    done = canvas.Tick().quit;
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  if (absl::GetFlag(FLAGS_run_demo)) {
    Demo();
  }
  return 0;
}
