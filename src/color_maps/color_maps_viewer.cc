#include "base/init.h"
#include "src/color_maps/color_maps.h"
#include "src/convert.h"
#include "src/image_viewer/image_viewer.h"

Image<PixelType::RGBAU8> MakeColorVec(ColorMap map, int size) {
  Image<PixelType::RGBAU8> out(size, 1);
  const VectorXd vals = VectorXd::LinSpaced(size, 0, 1.0);
  for (int i = 0; i < vals.size(); ++i) {
    out(i) = Convert<PixelType::RGBAU8>(GetMappedColor3f(map, vals[i]));
  }
  return out;
}

Image<PixelType::RGBAU8> MakeColorMapImage() {
  constexpr int kRows = 500;
  constexpr int kColsPerColor = 100;
  const int cols = kColsPerColor * kAllColorMaps.size();
  Image<PixelType::RGBAU8> data(kRows, cols);
  for (int i = 0; i < kAllColorMaps.size(); ++i) {
    auto color_vec = MakeColorVec(kAllColorMaps[i], kRows);
    data.block<kRows, kColsPerColor>(0, i * kColsPerColor) =
        color_vec.replicate<1, kColsPerColor>();
  }
  return data;
}

int main(int argc, char* argv[]) {
  Init(argc, argv);
  const auto data = MakeColorMapImage();
  ImageViewer viewer(data.cols(), data.rows());
  *viewer.data() = data;
  while (!viewer.Update().quit) {
  }
  return 0;
}
