#include <random>
#include <thread>

#include "base/format.h"
#include "base/init.h"
#include "base/scoped_profiler.h"
#include "graphics/animated_canvas.h"
#include "src/bresenham.h"
#include "src/color_maps/color_maps.h"
#include "src/convert.h"
#include "src/demo_utils.h"
#include "src/drawing_utils.h"
#include "src/emitter.h"
#include "src/fonts/font_renderer.h"
#include "src/mobile_object.h"
#include "src/random.h"
#include "src/scrolling_manager.h"

ABSL_FLAG(int32_t, emission_rate, 100, "Number of particles per second");

// Most of the major parts glued together
void RenderShip(const ScrollingManager& scroller, const Ship& ship,
                Image<PixelType::RGBAU8>* data) {
  Vector2i tail_start = ship.particle().state().head<2>().cast<int>();
  tail_start.y() -= scroller.viewport_bottom();
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

void RenderEnvironment(const BufferStack<Image<uint8_t>>& env,
                       const ScrollingManager& scroller,
                       Image<PixelType::RGBAU8>* data) {
  // Convert scalar environment pixel values into colors
  auto pixel_transform = [](uint8_t v) -> PixelType::RGBAU8 {
    if (v == kWall) {
      return kWallColor;
    } else {
      // Black
      return {0, 0, 0, 255};
    }
  };

  // Copy blocks of values out of the environment buffer, transform them into
  // the viewport space, and also transform the values into the color space.
  int viewport_bottom = 0;
  int start_row;
  int num_rows;
  const auto& buffers = env.buffers();
  for (int i = scroller.lowest_visible_buffer();
       i <= scroller.highest_visible_buffer(); ++i) {
    // Copy data
    scroller.VisibleRows(i, &start_row, &num_rows);
    data->block(viewport_bottom, 0, num_rows, data->cols()) =
        buffers[i]
            .block(start_row, 0, num_rows, data->cols())
            .unaryExpr(pixel_transform);
    viewport_bottom += num_rows;
  }
}

constexpr double kShipRotationRate = 10.0;
constexpr double kShipAcceleration = 200.0;
constexpr double kGravity = -kShipAcceleration / 4.0;

// Make a color look up table for the number of particles.
// The transition function allows you to use a nonlinear scale.
// for a linear scale just use a passthrough: [](double x) { return x;}
template <class T, ColorMap Map, int Size>
std::array<T, Size> MakeColorLut(
    const std::function<double(double)>& transition_function) {
  std::array<T, Size> out;
  for (int i = 0; i < Size; ++i) {
    const double p = transition_function(static_cast<double>(i) / (Size - 1));
    CHECK_GE(p, 0.0);
    CHECK_LE(p, 1.0);
    out[i] = Convert<T>(EvaluateColorMap<Map>(p));
  }
  return out;
}

// Particle motion, collision, environment destruction and particle rendering
void UpdateParticles(const double dt, const double ddy,
                     const ScrollingManager& scroller,
                     CircularBuffer<Vector5d>* particles,
                     BufferStack<Image<uint8_t>>* environment,
                     Image<uint8_t>* particle_density,
                     Image<PixelType::RGBAU8>* data) {
  particle_density->setConstant(0);
  const int num_particles = particles->Capacity();
  auto& mutable_particles = *particles->mutable_data();
  static constexpr int kMaxDensity = 32;
  for (int i = 0; i < num_particles; ++i) {
    auto& current = mutable_particles[i];
    if (current[4] /* ttl */ <= 0) {
      continue;
    }
    Vector2d pos;
    Vector2d vel;
    DestructingBresenham(current.segment<2>(0), current.segment<2>(2), dt, 1.0,
                         environment, &pos, &vel);
    current.segment<2>(0) = pos;
    current.segment<2>(2) = vel;
    current[3] += dt * ddy;
    current[4] -= dt;

    // Count particle
    Vector2i pos_i = pos.cast<int>();
    pos_i[1] -= scroller.viewport_bottom();
    if (pos_i.y() >= 0 && pos_i.y() < data->rows() && pos_i.x() >= 0 &&
        pos_i.x() < data->cols()) {
      if ((*particle_density)(pos_i[1], pos_i[0]) < (kMaxDensity - 1)) {
        (*particle_density)(pos_i[1], pos_i[0]) =
            std::min((*particle_density)(pos_i[1], pos_i[0]) + 1, kMaxDensity);
      }
    }
  }

  // Transform the particle density image into colors (This should be done on
  // the GPU)
  static std::array<PixelType::RGBAU8, kMaxDensity> color_lut =
      MakeColorLut<PixelType::RGBAU8, ColorMap::kMagma, kMaxDensity>(
          [](double x) {
            static constexpr double kMinColor = .1;
            static constexpr double kStartingSlope = 5;
            return std::tanh(kStartingSlope * x) * (1.0 - kMinColor) +
                   kMinColor;
          });

  for (int i = 0; i < particle_density->size(); ++i) {
    const auto& count = (*particle_density)(i);
    if (count > 0) {
      (*data)(i) = color_lut[count];
    }
  }
}

void MakeLevel(int i, std::mt19937* gen, Image<uint8_t>* level_buffer) {
  AddNoise(kWall, .2, gen, level_buffer);
  if (i <= 0) {
    AddBottomWall(kWall, level_buffer);
  }
  AddSideWalls(kWall, level_buffer);
}

void UpdateShip(const double dt, const ControllerInput& input,
                const BufferStack<Image<uint8_t>>& env, Ship* ship) {
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

void Demo(double emission_rate) {
  ScopedProfiler prof;
  // Set up canvas
  const double kFps = 60.0;
  const Vector2i window_dims(1366, 768);
  const Vector2i viewport_dims = window_dims / 2;
  std::mt19937 rando(0);
  AnimatedCanvas canvas(window_dims[0], window_dims[1], viewport_dims[0],
                        viewport_dims[1], kFps);
  int level_height = viewport_dims.y() * 2;

  // Set up environment
  auto make_next_level = [&rando](int level_num, Image<uint8_t>* data) {
    MakeLevel(level_num, &rando, data);
  };
  ScrollingCanvas<uint8_t> scrolling_canvas(level_height, viewport_dims[0],
                                            viewport_dims[1],
                                            std::move(make_next_level));
  const auto& scroller = scrolling_canvas.scrolling_manager();
  const auto& environment = scrolling_canvas.tiles();
  Image<uint8_t> particle_density(viewport_dims[1], viewport_dims[0]);

  // Make emitter
  const double angular_stdev = .15;
  const double min_speed = 150.0;
  const double max_speed = 200.0;
  const double min_life = 1;
  const double max_life = 2;
  Emitter e(angular_stdev, min_speed, max_speed, emission_rate, min_life,
            max_life);

  // Set up ship.
  auto ship_start = FindEmptySpot(environment.buffers().front());
  CHECK(ship_start) << "Environment is full?";
  Vector2d init_pos = ship_start->cast<double>() + Vector2d(.5, .5);
  Ship ship(MobileObject({init_pos.x(), init_pos.y(), 0, 0, 0}), M_PI / 2.0);

  SO2d previous_orientation = Rotate180(ship.orientation());
  Vector5d previous_state = ship.particle().state();

  auto* data = canvas.data();
  Duration dt(0);
  ControllerInput input;
  while (!input.quit) {
    const double n_seconds = ToSeconds<double>(dt);
    const auto& current_orientation = Rotate180(ship.orientation());
    const auto& current_state = ship.particle().state();
    if (input.up) {
      e.EmitOverTime(n_seconds, previous_orientation, current_orientation,
                     previous_state.head<2>(), current_state.head<2>(),
                     previous_state.segment<2>(2), current_state.segment<2>(2));
    }
    UpdateShip(n_seconds, input, environment, &ship);

    // Update the viewport to respond to changes in the ships position.
    int viewport_mid = (scroller.viewport_bottom() + viewport_dims[1] / 2);
    int ship_row = static_cast<int>(ship.particle().state().y());
    scrolling_canvas.Scroll(ship_row - viewport_mid);

    RenderEnvironment(environment, scroller, data);
    previous_orientation = current_orientation;
    previous_state = current_state;
    UpdateParticles(n_seconds, kGravity, scroller, e.mutable_particles(),
                    scrolling_canvas.mutable_tiles(), &particle_density, data);
    RenderShip(scroller, ship, data);
    AddFpsText(canvas.fps(), text_color, data);
    input = canvas.Tick(&dt);
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Demo(absl::GetFlag(FLAGS_emission_rate));
  return 0;
}
