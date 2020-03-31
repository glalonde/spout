#include <random>
#include <thread>

#include "base/format.h"
#include "base/init.h"
#include "src/bresenham.h"
#include "src/demo_utils.h"
#include "src/drawing_utils.h"
#include "graphics/animated_canvas.h"
#include "src/mobile_object.h"

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
  std::mt19937 gen(0);
  AddNoise(kWall, .2, &gen, &environment);
  AddAllWalls(kWall, &environment);

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
