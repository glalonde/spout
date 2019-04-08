#include <random>

#include "base/format.h"
#include "base/init.h"
#include "base/wall_timer.h"
#include "src/color_maps/color_maps.h"
#include "src/convert.h"
#include "src/image_viewer/animated_canvas.h"
#include "src/random.h"
#include "src/scrolling_manager.h"

void MakePerlinNoiseLevel(int i /* level number */, std::mt19937* rand_gen,
                          Image<PixelType::RGBAU8>* level_buffer) {
  const ColorMap color_map = kAllColorMaps[i % kAllColorMaps.size()];
  Image<double> perlin_vals(level_buffer->rows(), level_buffer->cols());
  PerlinNoise(0.0, 1.0, level_buffer->cols() / 5, rand_gen, perlin_vals);
  for (int r = 0; r < level_buffer->rows(); ++r) {
    for (int c = 0; c < level_buffer->cols(); ++c) {
      const auto color = Convert<PixelType::RGBAU8>(
          GetMappedColor3f(color_map, perlin_vals(r, c)));
      (*level_buffer)(r, c) = color;
    }
  }
}

void Demo() {
  // Set up canvas
  const double kFps = 60.0;
  const Vector2i window_dims(640, 480);
  const Vector2i grid_dims = window_dims / 4;
  LOG(INFO) << "Window dims: " << window_dims.transpose();
  LOG(INFO) << "Grid dims: " << grid_dims.transpose();
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], kFps);

  std::mt19937 rand_gen(0);
  auto level_gen = [rand_gen = std::move(rand_gen)](
                       int level_number,
                       Image<PixelType::RGBAU8>* level_buffer) mutable {
    MakePerlinNoiseLevel(level_number, &rand_gen, level_buffer);
  };

  // Viewport is half of a level size, so we will be able to see 1 or 2 level
  // buffers at a time only.
  int level_height = grid_dims.y() * 2;
  ScrollingCanvas scroller({grid_dims.x(), level_height}, grid_dims[1],
                           std::move(level_gen));
  LOG(INFO) << "Level height: " << level_height;

  // Loop
  auto* data = canvas.data();
  double screen_bottom_height = 0;
  constexpr double kScrollRate = 200.0 /* rows per second */;

  int actual_screen_height;
  constexpr double kMinHeight = 0;
  constexpr double kMaxHeight = std::numeric_limits<int>::max() / 2.0;

  ControllerInput input;
  Duration tick_time;
  while (!input.quit) {
    // Render current state, collect input to process during the next tick.
    scroller.Render(data);
    input = canvas.Tick(&tick_time);
    // Process input recieved in that tick.
    if (input.up && !input.down) {
      screen_bottom_height += kScrollRate * ToSeconds<double>(tick_time);
    } else if (input.down && !input.up) {
      screen_bottom_height -= kScrollRate * ToSeconds<double>(tick_time);
    }
    screen_bottom_height =
        std::clamp(screen_bottom_height, kMinHeight, kMaxHeight);
    actual_screen_height = static_cast<int>(screen_bottom_height);
    scroller.SetHeight(actual_screen_height);
  }
}

int main(int argc, char** argv) {
  Init(argc, argv);
  Demo();
  return 0;
}
