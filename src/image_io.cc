#include "src/image_io.h"
#include "base/logging.h"
#define STB_IMAGE_IMPLEMENTATION
#include <stb_image.h>

std::optional<Image<PixelType::RGBAU8>> ReadImage(const std::string& path) {
  int width, height, n_channels;
  constexpr int kDesiredChannels = PixelType::RGBAU8::RowsAtCompileTime;
  unsigned char* data =
      stbi_load(path.c_str(), &width, &height, &n_channels, kDesiredChannels);
  if (!data) {
    return {};
  }
  Eigen::Map<
      Eigen::Array<uint8_t, Eigen::Dynamic, Eigen::Dynamic, Eigen::RowMajor>>
      mapped_data(data, height, width * kDesiredChannels);
  Image<PixelType::RGBAU8> out(height, width);
  for (int i = 0; i < height; ++i) {
    for (int j = 0; j < width; ++j) {
      out(i, j) =
          mapped_data.block<1, kDesiredChannels>(i, j * kDesiredChannels);
    }
  }
  stbi_image_free(data);
  return out;
}
