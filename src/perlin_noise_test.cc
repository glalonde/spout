#include "src/perlin_noise.h"
#include <Eigen/Core>
#include "base/googletest.h"
#include "src/color_maps/color_maps.h"
#include "src/convert.h"
#include "src/image_viewer/image_viewer.h"

GTEST_TEST(PerlinNoise, Smoke) {
  std::mt19937 gen(0);
  auto gradients = GradientArray(5, 5, &gen);
  for (const auto& v : gradients.reshaped()) {
    LOG(INFO) << v;
  }
}

GTEST_TEST(PerlinNoise, Viewer) {
  int cols = 640;
  int rows = 480;

  ImageViewer viewer(cols, rows);
  viewer.SetTextureSize(cols, rows);
  auto* data = viewer.data();

  Image<double> perlin_vals(rows, cols);
  std::mt19937 gen(0);
  PerlinNoise(10, &gen, perlin_vals);
  perlin_vals *= .5;
  perlin_vals += .5;

  const ColorMap color_map = ColorMap::kParula;
  for (int r = 0; r < rows; ++r) {
    for (int c = 0; c < cols; ++c) {
      const auto color = Convert<PixelType::RGBAU8>(
          GetMappedColor3f(color_map, perlin_vals(r, c)));
      (*data)(r, c) = color;
    }
  }

  while (!viewer.Update().quit) {
  }
}

GTEST_MAIN();
