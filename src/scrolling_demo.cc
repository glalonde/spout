#include <random>

#include "base/format.h"
#include "base/init.h"
#include "base/wall_timer.h"
#include "src/color_maps/color_maps.h"
#include "src/convert.h"
#include "src/image_viewer/animated_canvas.h"
#include "src/random.h"
#include "src/scrolling_manager.h"

// Viewport width == level width
class ScrollingCanvas {
 public:
  ScrollingCanvas(Vector2i level_dimensions /* width, height */,
                  int viewport_height)
      : level_dimensions_(level_dimensions),
        manager_(level_dimensions.y(), viewport_height) {}

  void SetHeight(int screen_bottom) {
    manager_.UpdateHeight(screen_bottom);
  }

  void Render(Image<PixelType::RGBAU8>* viewport) {
    while (manager_.highest_visible_buffer() + 1 >= buffers_.size()) {
      MakeLevelBuffer(buffers_.size());
    }
    int viewport_bottom = 0;
    int start_row;
    int num_rows;
    for (int i = manager_.lowest_visible_buffer();
         i <= manager_.highest_visible_buffer(); ++i) {
      // Copy data
      manager_.VisibleRows(i, &start_row, &num_rows);
      viewport->block(viewport_bottom, 0, num_rows, viewport->cols()) =
          buffers_[i].block(start_row, 0, num_rows, viewport->cols());
      viewport_bottom += num_rows;
    }
  }

 private:
  void MakeLevelBuffer(int i) {
    LOG(INFO) << "Computing level: " << i;
    CHECK_EQ(buffers_.size(), i);
    buffers_.emplace_back(level_dimensions_.y(), level_dimensions_.x());
    const ColorMap color_map = kAllColorMaps[i % kAllColorMaps.size()];
    Image<double> perlin_vals(level_dimensions_.y(), level_dimensions_.x());
    PerlinNoise(0.0, 1.0, level_dimensions_.x() / 5, &rand_gen_, perlin_vals);
    for (int r = 0; r < level_dimensions_.y(); ++r) {
      for (int c = 0; c < level_dimensions_.x(); ++c) {
        const auto color = Convert<PixelType::RGBAU8>(
            GetMappedColor3f(color_map, perlin_vals(r, c)));
        buffers_.back()(r, c) = color;
      }
    }
  }

  Vector2i level_dimensions_;
  std::mt19937 rand_gen_;
  ScrollingManager manager_;
  std::vector<Image<PixelType::RGBAU8>> buffers_;
};

void Demo() {
  // Set up canvas
  const double kFps = 60.0;
  const Vector2i window_dims(640, 480);
  const Vector2i grid_dims = window_dims / 4;
  LOG(INFO) << "Window dims: " << window_dims.transpose();
  LOG(INFO) << "Grid dims: " << grid_dims.transpose();
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], kFps);

  // Viewport is half of a level size, so we will be able to see 1 or 2 level
  // buffers at a time only.
  int level_height = grid_dims.y() * 2;
  ScrollingCanvas scroller({grid_dims.x(), level_height}, grid_dims[1]);
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
