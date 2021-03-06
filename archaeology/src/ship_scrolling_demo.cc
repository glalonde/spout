#include <random>
#include <thread>

#include "base/format.h"
#include "base/init.h"
#include "graphics/animated_canvas.h"
#include "src/bresenham.h"
#include "src/demo_utils.h"
#include "src/drawing_utils.h"
#include "src/mobile_object.h"
#include "src/scrolling_manager.h"

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

constexpr double kShipRotationRate = 15.0;
constexpr double kShipAcceleration = 200.0;
constexpr double kGravity = -kShipAcceleration / 5.0;

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

void MakeLevel(int i, std::mt19937* gen, Image<uint8_t>* level_buffer) {
  AddNoise(kWall, .2, gen, level_buffer);
  if (i <= 0) {
    AddBottomWall(kWall, level_buffer);
  }
  AddSideWalls(kWall, level_buffer);
}

void RenderEnvironment(const BufferStack<Image<uint8_t>>& env,
                       const ScrollingManager& scroller,
                       Image<PixelType::RGBAU8>* data) {
  auto pixel_transform = [](uint8_t v) -> PixelType::RGBAU8 {
    if (v == kWall) {
      return kWallColor;
    } else {
      return {0, 0, 0, 255};
    }
  };

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

void Demo() {
  // Set up canvas
  const double kFps = 60.0;
  const Vector2i window_dims(800, 800);
  const Vector2i viewport_dims = window_dims / 4;
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

  // Set up ship.
  auto ship_start = FindEmptySpot(environment.buffers().front());
  CHECK(ship_start) << "Environment is full?";
  Vector2d init_pos = ship_start->cast<double>() + Vector2d(.5, .5);
  Ship ship(MobileObject({init_pos.x(), init_pos.y(), 0, 0, 0}), M_PI / 2.0);

  // Rendering target
  auto* data = canvas.data();
  ControllerInput input;
  Duration tick_time;
  while (!input.quit) {
    // Update the viewport to respond to changes in the ships position.
    int viewport_mid = (scroller.viewport_bottom() + viewport_dims[1] / 2);
    int ship_row = static_cast<int>(ship.particle().state().y());
    scrolling_canvas.Scroll(ship_row - viewport_mid);

    // Render
    RenderEnvironment(environment, scroller, data);
    AddFpsText(canvas.fps(), text_color, data);
    RenderShip(scroller, ship, data);
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
