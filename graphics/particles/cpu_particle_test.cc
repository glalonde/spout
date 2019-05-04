#include "base/format.h"
#include "base/init.h"
#include "graphics/animated_canvas.h"
#include "src/bresenham.h"
#include "src/convert.h"
#include "src/demo_utils.h"
#include "src/fonts/font_renderer.h"
#include "src/random.h"
#include "src/so2.h"

DEFINE_int32(num_particles, 512, "Number of particles");
DEFINE_double(damage_rate, 1.0, "Damage rate");
DEFINE_double(dt, .016, "Simulation rate");

static constexpr int kMantissaBits = 8;

void RenderParticle(const Vector2u32& pos, Image<PixelType::RGBAU8>* data) {
  auto get_cell = [](const Vector2u32& vec) -> Vector2i {
    return vec.unaryExpr([](uint32_t v) -> int {
      return static_cast<int>(GetLowRes<kMantissaBits>(v)) -
             kAnchor<uint32_t, kMantissaBits>;
    });
  };
  // (x, y) -> (col, height - row)
  Vector2i pos_i = get_cell(pos);
  (*data)(pos_i[1], pos_i[0]) = kParticleColor;
}

struct IntParticle {
  Vector2<uint32_t> position;
  Vector2<int32_t> velocity;
};

static constexpr int32_t kDenseWall = 1000;

void MakeLevel(std::mt19937* gen, Image<int32_t>* level_buffer) {
  AddNoise(kDenseWall, .5, gen, level_buffer);
  AddAllWalls(kDenseWall, level_buffer);
}

void SetRandomPoints(Vector2i start_cell, int cell_size,
                     Vector<IntParticle, Eigen::Dynamic>* data) {
  std::mt19937 gen(0);
  auto magnitude_dist =
      UniformRandomDistribution<double>(400 * cell_size, 500 * cell_size);
  auto angle_dist = UniformRandomDistribution<double>(-M_PI, M_PI);
  for (int i = 0; i < data->size(); ++i) {
    (*data)[i].position = Vector2u32::Constant(
        SetLowRes<kMantissaBits>(kAnchor<uint32_t, kMantissaBits>));
    (*data)[i].position += (start_cell * cell_size).cast<uint32_t>();
    (*data)[i].velocity =
        (SO2d(angle_dist(gen)).data() * magnitude_dist(gen)).cast<int>();
  }
}

void TestLoop(int num_particles) {
  int window_width = 1440;
  int window_height = 900;
  int pixel_size = 4;
  int grid_width = window_width / pixel_size;
  int grid_height = window_height / pixel_size;

  // Set up canvas
  const double kFps = 60.0;
  const int cell_size = kCellSize<uint32_t, kMantissaBits>;
  AnimatedCanvas canvas(window_width, window_height, grid_width, grid_height,
                        kFps);

  std::mt19937 rand_gen(0);

  // Set up environment
  Image<int32_t> environment(grid_height, grid_width);
  environment.setConstant(0);
  MakeLevel(&rand_gen, &environment);

  // Set up particles
  Eigen::Vector<IntParticle, Eigen::Dynamic> particles(num_particles);
  SetRandomPoints(Vector2i(grid_width / 2, grid_height / 2), cell_size,
                  &particles);

  IntParticle next;
  double dt = FLAGS_dt;
  bool done = false;
  auto* data = canvas.data();
  while (!done) {
    RenderEnvironment(environment, data);
    for (int i = 0; i < num_particles; ++i) {
      BresenhamExperimentLowResDestructive(
          particles[i].position, particles[i].velocity, dt, FLAGS_damage_rate,
          &environment, &next.position, &next.velocity);
      particles[i] = next;
      LOG(INFO) << next.position.transpose() << ", " << next.velocity.transpose();
      RenderParticle(next.position, data);
    }
    AddFpsText(canvas.fps(), text_color, data);
    done = canvas.Tick().quit;
  }
}

int main(int argc, char* argv[]) {
  Init(argc, argv);
  TestLoop(FLAGS_num_particles);
  return 0;
}
