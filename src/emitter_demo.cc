#include <thread>
#include <random>

#include "base/format.h"
#include "base/init.h"
#include "graphics/animated_canvas.h"
#include "src/bresenham.h"
#include "src/convert.h"
#include "src/demo_utils.h"
#include "src/emitter.h"
#include "src/fonts/font_renderer.h"
#include "src/drawing_utils.h"
#include "src/random.h"
#include "src/mobile_object.h"

ABSL_FLAG(int32_t, emission_rate, 100, "Number of particles per second");

void RenderParticle(const Vector2d& pos, Image<PixelType::RGBAU8>* data) {
  Vector2i pos_i = pos.cast<int>();
  (*data)(pos_i[1], pos_i[0]) = kParticleColor;
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

void UpdateParticles(const double dt, const double ddy,
                     CircularBuffer<Vector5d>* particles,
                     Image<uint8_t>* environment,
                     Image<PixelType::RGBAU8>* data) {
  const int num_particles = particles->Capacity();
  auto& mutable_particles = *particles->mutable_data();
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
    RenderParticle(current.segment<2>(0), data);
  }
}

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

void Demo(double emission_rate) {
  // Set up canvas
  const double kFps = 60.0;
  const Vector2i window_dims(640, 480);
  const Vector2i grid_dims = window_dims / 4;
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], kFps);

  std::mt19937 rand_gen(0);

  // Set up environment
  Image<uint8_t> environment(grid_dims[1], grid_dims[0]);
  environment.setConstant(0);
  AddNoise(kWall, .2, &rand_gen, &environment);
  AddAllWalls(kWall, &environment);

  // Make emitter
  const double angular_stdev = .2;
  const double min_speed = 30.0;
  const double max_speed = 70.0;
  const double min_life = 2;
  const double max_life = 3;
  Emitter e(angular_stdev, min_speed, max_speed, emission_rate, min_life,
            max_life);

  // Set up ship.
  auto ship_start = FindEmptySpot(environment);
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
    RenderEnvironment(environment, data);
    UpdateShip(n_seconds, input, environment, &ship);
    previous_orientation = current_orientation;
    previous_state = current_state;
    RenderShip(ship, data);
    UpdateParticles(n_seconds, kGravity, e.mutable_particles(), &environment,
                    data);
    AddFpsText(canvas.fps(), text_color, data);
    input = canvas.Tick(&dt);
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Demo(absl::GetFlag(FLAGS_emission_rate));
  return 0;
}
