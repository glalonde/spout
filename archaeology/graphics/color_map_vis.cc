#include <thread>
#include "base/format.h"
#include "base/init.h"
#include "base/scoped_profiler.h"
#include "base/wall_timer.h"
#include "src/color_maps/color_maps.h"
#include "src/convert.h"
#include "src/fonts/font_renderer.h"
#include "graphics/animated_canvas.h"

ABSL_FLAG(int32_t, width, 256, "display width");
ABSL_FLAG(int32_t, height, 256, "display height");

ABSL_FLAG(double, texture_scale, 1.0, "texture scale factor");
ABSL_FLAG(int32_t, color_map_index, 0, "Color map index, see color_maps.h");

// Set the data in the image to a radial gradient
void SetToGradient(const ColorMap map, double phase,
                   Image<PixelType::RGBAU8>* data) {
  // (row, col)
  const Vector2d center = Vector2d(data->rows(), data->cols()) / 2.0;
  const double radius = center.norm();
  // (row, col)
  Vector2d current;
  for (int c = 0; c < data->cols(); ++c) {
    current[1] = c;
    for (int r = 0; r < data->rows(); ++r) {
      current[0] = r;
      const double normalized_distance = (center - current).norm() / radius;
      const double map_index = std::fmod(normalized_distance + phase, 1.0);
      const Vector3f color = GetMappedColor3f(map, map_index);
      (*data)(r, c) = Convert<PixelType::RGBAU8, PixelType::RGBF32>(color);
    }
  }
}

void AddFpsText(double fps, const PixelType::RGBAU8& color,
                Image<PixelType::RGBAU8>* data) {
  std::string fps_string = FormatString("%.0f", fps);
  RenderString(fps_string, {1, 1}, color, 1,
               font_rendering::Justification::kLeft, data);
}

int main(int argc, char* argv[]) {
  Init(argc, argv);
  const int tex_width =
      absl::GetFlag(FLAGS_width) * absl::GetFlag(FLAGS_texture_scale);
  const int tex_height =
      absl::GetFlag(FLAGS_height) * absl::GetFlag(FLAGS_texture_scale);
  AnimatedCanvas canvas(absl::GetFlag(FLAGS_width), absl::GetFlag(FLAGS_height),
                        tex_width, tex_height, 60.0);
  ScopedProfiler prof;

  PixelType::RGBAU8 text_color =
      Convert<PixelType::RGBAU8>(PixelType::RGBF64(1.0, 0.0, 0.0));
  auto* data = canvas.data();
  CHECK_GE(absl::GetFlag(FLAGS_color_map_index), 0);
  CHECK_LT(absl::GetFlag(FLAGS_color_map_index), kAllColorMaps.size());
  bool done = false;
  WallTimer timer;
  timer.Start();
  while (!done) {
    SetToGradient(kAllColorMaps[absl::GetFlag(FLAGS_color_map_index)],
                  ToSeconds<double>(timer.ElapsedDuration()), data);
    AddFpsText(canvas.fps(), text_color, data);
    done = canvas.Tick().quit;
  }
  return 0;
}
