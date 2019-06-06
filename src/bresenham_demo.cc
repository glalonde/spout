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

ABSL_FLAG(int32_t, num_particles, 100, "Number of particles");

void RenderParticle(const Vector2u32& pos, Image<PixelType::RGBAU8>* data) {
  auto get_cell = [](const Vector2u32& vec) -> Vector2i {
    return vec.unaryExpr([](uint32_t v) -> int {
      return static_cast<int>(GetLowRes<8>(v)) - kAnchor<uint32_t, 8>;
    });
  };
  // (x, y) -> (col, height - row)
  Vector2i pos_i = get_cell(pos);
  (*data)(pos_i[1], pos_i[0]) = kParticleColor;
}

void Demo(int num_particles) {
  // Set up canvas
  const double kFps = 60.0;
  const int cell_size = kCellSize<uint32_t, 8>;
  const Vector2i window_dims(800, 800);
  const Vector2i grid_dims = window_dims / 4;
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], kFps);

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

  bool done = false;
  auto* data = canvas.data();
  while (!done) {
    RenderEnvironment(environment, data);
    for (int i = 0; i < num_particles; ++i) {
      BresenhamExperimentLowRes(particles[i].first, particles[i].second, dt,
                                environment, &pos, &vel);
      particles[i].first = pos;
      particles[i].second = vel;
      particles[i].second[1] += static_cast<int>(dt * ddy);
      RenderParticle(pos, data);
    }
    AddFpsText(canvas.fps(), text_color, data);
    done = canvas.Tick().quit;
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Demo(absl::GetFlag(FLAGS_num_particles));
  return 0;
}
