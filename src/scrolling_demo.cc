#include <random>

#include "base/format.h"
#include "base/init.h"
#include "base/wall_timer.h"
#include "src/image_viewer/animated_canvas.h"
#include "src/scrolling_manager.h"

class ScrollingCanvas {
 public:
  ScrollingCanvas(int buffer_height, int viewport_height)
      : manager_(buffer_height, viewport_width) {}

  void SetHeight(int screen_bottom) {
    manager_.UpdateHeight(screen_bottom);
  }

  void Render(Image<PixelType::RGBAU8>* viewport) {
    for (int i = manager_.lowest_visible_buffer();
         i <= manager_.highest_visible_buffer(); ++i) {
      LOG(INFO) << "Showing buffer: " << i;
    }
  }

 private:
  ScrollingManager manager_;
  std::vector<Image<PixelType::RGBAU8>> buffers_;
};

void Demo() {
  // Set up canvas
  const double kFps = 60.0;
  const Vector2i window_dims(800, 800);
  const Vector2i grid_dims = window_dims / 4;
  AnimatedCanvas canvas(window_dims[0], window_dims[1], grid_dims[0],
                        grid_dims[1], kFps);

  // Viewport is half of a level size, so we will be able to see 1 or 2 level
  // buffers at a time only.
  int level_height = grid_dims.y() * 2;
  ScrollingCanvas scroller(level_height, grid_dims[1]);

  // Loop
  bool done = false;
  auto* data = canvas.data();
  double screen_bottom_height = 0;
  constexpr double kScrollRate = 100.0 /* rows per second */;

  int actual_screen_height;
  constexpr double kMinHeight = 0;
  constexpr double kMaxHeight = std::numeric_limit<int>::max() / 2.0;

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
