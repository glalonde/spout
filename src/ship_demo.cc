#include <random>
#include <thread>

#include "base/format.h"
#include "base/init.h"
#include "src/bresenham.h"
#include "src/convert.h"
#include "src/fonts/font_renderer.h"
#include "src/image_viewer/animated_canvas.h"
#include "src/mobile_object.h"
#include "src/random.h"
#include "src/drawing_utils.h"

DEFINE_int32(num_particles, 100, "Number of particles");

static constexpr uint8_t kWall = std::numeric_limits<uint8_t>::max();
static const PixelType::RGBAU8 kWallColor = {0, 0, 255, 255};
static const PixelType::RGBAU8 kParticleColor = {0, 255, 0, 255};
static const PixelType::RGBAU8 kShipColor = {255, 255, 0, 255};
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

// Returns the min corner (row, col) and the (rows, cols) sizes of the smallest
// concentric ring that is nested inside the given dimensions `rows_cols`
void SmallestConcentricRing(const Vector2i& rows_cols, Vector2i* min_corner,
                            Vector2i* sizes) {
  const auto& rows = rows_cols[0];
  const auto& cols = rows_cols[1];
  if (rows <= cols) {
    (*sizes)[0] = rows % 2;
    (*sizes)[1] = cols - rows + (*sizes)[0];
  } else {
    (*sizes)[1] = cols % 2;
    (*sizes)[0] = rows - cols + (*sizes)[1];
  }
  (*min_corner) = rows_cols / 2 - (*sizes) / 2;
}

// Returns the coordinates of the first nonzero cell spiraling out of the
// center, or nullopt if the whole thing is full.
std::optional<Vector2i> FindEmptySpot(const Image<uint8_t>& env) {
  Vector2i sizes;
  Vector2i min_corner;
  Vector2i rows_cols(env.rows(), env.cols());
  SmallestConcentricRing(rows_cols, &min_corner, &sizes);
  while (sizes[0] <= env.rows() && sizes[1] <= env.cols()) {
    Vector2i max_corner = min_corner.array() + (sizes.array() - 1);
    // min and max cols
    for (int r = min_corner[0]; r <= max_corner[0]; ++r) {
      if (env(r, min_corner[1]) <= 0) {
        return Vector2i(r, min_corner[1]);
      } else if (env(r, max_corner[1]) <= 0) {
        return Vector2i(r, max_corner[1]);
      }
    }
    // min and max rows
    for (int c = min_corner[1]; c <= max_corner[1]; ++c) {
      if (env(min_corner[0], c) <= 0) {
        return Vector2i(min_corner[0], c);
      } else if (env(max_corner[0], c) <= 0) {
        return Vector2i(max_corner[0], c);
      }
    }
    sizes.array() += 2;
    min_corner.array() -= 1;
  }
  return {};
}

void RenderShip(const Ship& ship, Image<PixelType::RGBAU8>* data) {
  Vector2i tail_start = ship.particle().state().head<2>().cast<int>();
  static constexpr double kTailLength = -10.0;
  static constexpr double kShipAngle = M_PI / 5.0;
  static const SO2d kHalfShipAngle(kShipAngle / 2.0);
  const Vector2i tail_end0 =
      tail_start +
      (kTailLength * (ship.orientation() * kHalfShipAngle).data()).cast<int>();
  const Vector2i tail_end1 =
      tail_start +
      (kTailLength * (ship.orientation() * kHalfShipAngle.inverse()).data())
          .cast<int>();
  DrawLine(tail_start.x(), tail_start.y(), tail_end0.x(), tail_end0.y(),
           kShipColor, data);
  DrawLine(tail_start.x(), tail_start.y(), tail_end1.x(), tail_end1.y(),
           kShipColor, data);
}

constexpr double kShipRotationRate = 15.0;
constexpr double kShipAcceleration = 200.0;
constexpr double kGravity = -kShipAcceleration / 5.0;

void UpdateShip(const double dt, const ControllerInput& input,
                const Image<uint8_t>& env, Ship* ship) {
  // Updates velocity and time to live.
  DeltaParticle dp = DeltaParticle(0, kGravity, -1.0) * dt;
  ship->mutable_particle()->ApplyDelta(dp);

  // Handle inputs
  if (input.up && !input.down) {
    // Accelerate
    ship->Accelerate(dt * kShipAcceleration);
  }

  if (input.right && !input.left) {
    // Rotate Clockwise
    ship->Rotate(-kShipRotationRate * dt);
  }
  if (input.left && !input.right) {
    // Rotate CCW
    ship->Rotate(kShipRotationRate * dt);
  }

  // Manage collisions with environment.
  const auto& particle = ship->particle().state();
  Vector2d new_pos;
  Vector2d new_vel;
  SubPixelBresenhamNormal(particle.head<2>() /* pos */,
                          particle.segment<2>(2) /* vel */, dt, env, &new_pos,
                          &new_vel);
  auto& mutable_particle = *(ship->mutable_particle()->mutable_state());
  mutable_particle.head<2>() = new_pos;
  mutable_particle.segment<2>(2) = new_vel;
}

void Demo() {
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

  // Set up ship.
  auto ship_start = FindEmptySpot(environment);
  CHECK(ship_start) << "Environment is full?";
  Vector2d init_pos = ship_start->cast<double>() + Vector2d(.5, .5);
  Ship ship(MobileObject({init_pos.x(), init_pos.y(), 0, 0, 0}), M_PI / 2.0);

  auto* data = canvas.data();
  ControllerInput input;
  Duration tick_time;
  while (!input.quit) {
    // Render
    RenderEnvironment(environment, data);
    AddFpsText(canvas.fps(), text_color, data);
    RenderShip(ship, data);
    input = canvas.Tick(&tick_time);
    // Update game logic
    UpdateShip(ToSeconds<double>(tick_time), input, environment, &ship);
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Demo();
  return 0;
}
