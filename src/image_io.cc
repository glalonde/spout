#include "src/image_io.h"
#include "base/logging.h"
#define STB_IMAGE_IMPLEMENTATION
#define STB_IMAGE_WRITE_IMPLEMENTATION
#include <stb_image.h>
#include <stb_image_write.h>

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

void WriteImage(const Image<PixelType::RGBAU8>& image,
                const std::string& path) {
  constexpr int kNumChannels = PixelType::RGBAU8::RowsAtCompileTime;
  // Swap col-major to row-major
  Eigen::Array<PixelType::RGBAU8, Eigen::Dynamic, Eigen::Dynamic,
               Eigen::RowMajor>
      swapped = image;
  const int stride = sizeof(PixelType::RGBAU8) * swapped.cols();
  stbi_write_png_compression_level = 20;
  int result = stbi_write_png(path.c_str(), swapped.cols(), swapped.rows(),
                              kNumChannels, swapped.data(), stride);
  if (result != 0) {
    LOG(ERROR) << "Failed to write image to " << path
               << ", return code: " << result;
  }
}
