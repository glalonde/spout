#include <thread>
#include "base/init.h"
#include "base/time.h"
#include "src/color_maps/color_maps.h"
#include "src/convert.h"
#include "src/image_viewer/image_viewer.h"

DEFINE_int32(width, 256, "display width");
DEFINE_int32(height, 256, "display height");
DEFINE_int32(color_map_index, 0, "Color map index, see color_maps.h");

// Set the data in the image to a radial gradient
void SetToGradient(const ColorMap map, Image<PixelType::RGBAU8>* data) {
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
      const Vector3f color = GetMappedColor3f(map, normalized_distance);
      (*data)(r, c) = Convert<PixelType::RGBAU8, PixelType::RGBF32>(color);
    }
  }
}

int main(int argc, char* argv[]) {
  Init(argc, argv);
  ImageViewer viewer(FLAGS_width, FLAGS_height);
  auto* data = viewer.data();
  CHECK_GE(FLAGS_color_map_index, 0);
  CHECK_LT(FLAGS_color_map_index, kAllColorMaps.size());
  SetToGradient(kAllColorMaps[FLAGS_color_map_index], data);
  while (!viewer.Update().quit) {
  }
  return 0;
}
