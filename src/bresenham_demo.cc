#include <thread>
#include <random>

#include "base/format.h"
#include "base/init.h"
#include "src/bresenham.h"
#include "src/convert.h"
#include "src/fonts/font_renderer.h"
#include "graphics/animated_canvas.h"
#include "src/random.h"

DEFINE_int32(num_particles, 100, "Number of particles");

static constexpr uint8_t kWall = std::numeric_limits<uint8_t>::max();
static const PixelType::RGBAU8 kWallColor = {0, 0, 255, 255};
static const PixelType::RGBAU8 kParticleColor = {0, 255, 0, 255};
static const PixelType::RGBAU8 kTrailColor = {0, 128, 0, 255};
static const PixelType::RGBAU8 text_color =
    Convert<PixelType::RGBAU8>(PixelType::RGBF64(1.0, 0.0, 0.0));

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
  pos_i[1] = pos_i[1];
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

void AddFpsText(double fps, const PixelType::RGBAU8& color,
                Image<PixelType::RGBAU8>* data) {
  std::string fps_string = FormatString("%.0f", fps);
  RenderString(fps_string, {1, 1}, color, 1,
               font_rendering::Justification::kLeft, data);
}

template <class T>
void AddNoise(const T& wall_value, double percent_filled, Image<T>* data) {
  std::mt19937 gen(0);
  Image<double> perlin_vals(data->rows(), data->cols());
  PerlinNoise(0.0, 1.0, data->cols() / 10, &gen, perlin_vals);
  (*data) = perlin_vals.unaryExpr(
      [percent_filled, wall_value](double noise_val) -> T {
        if (noise_val <= percent_filled) {
          return wall_value;
        } else {
          return T(0);
        }
      });
}

void Demo(int num_particles) {
  // Set up canvas
  const double kFps = 60.0;
  const Vector2i window_dims(800, 800);
  const Vector2i grid_dims = window_dims / 4;
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], kFps);

  // Set up environment
  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  AddNoise(kWall, .2, &environment);
  AddWalls(kWall, &environment);

  // Set up particles
  AlignedBox<double, 4> particle_space;
  {
    particle_space.max() << 50, 50, 30, 30;
    particle_space.min() << 50, 50, -30, -30;
  }
  std::vector<Vector4d> particles(num_particles);
  for (int i = 0; i < num_particles; ++i) {
    particles[i] = particle_space.sample();
  }

  Vector2d pos;
  Vector2d vel;
  double dt = ToSeconds<double>(FromHz(kFps));
  const double ddy = -9.81;

  bool done = false;
  auto* data = canvas.data();
  while (!done) {
    RenderEnvironment(environment, data);
    for (int i = 0; i < num_particles; ++i) {
      DestructingBresenham(particles[i].segment<2>(0),
                           particles[i].segment<2>(2), dt, 1.0, &environment,
                           &pos, &vel);
      particles[i].segment<2>(0) = pos;
      particles[i].segment<2>(2) = vel;
      particles[i][3] += dt * ddy;
      RenderParticle(pos, data);
    }
    AddFpsText(canvas.fps(), text_color, data);
    done = canvas.Tick().quit;
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Demo(FLAGS_num_particles);
  return 0;
}
