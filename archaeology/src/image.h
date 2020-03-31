#pragma once
#include "src/eigen_types.h"

template <class PixelType>
using Image =
    Eigen::Array<PixelType, Eigen::Dynamic, Eigen::Dynamic, Eigen::RowMajor>;

namespace PixelType {
using GrayU8 = Vector1<uint8_t>;
using RGBU8 = Vector3<uint8_t>;
using RGBAU8 = Vector4<uint8_t>;
using RGBF32 = Vector3<float>;
using RGBF64 = Vector3<double>;
using RGBAF32 = Vector4<float>;
using RGBAF64 = Vector4<double>;
}  // namespace PixelType


// Check if row and col is a valid pixel on the image
template<class T>
bool IsInImage(int row, int col, const Image<T>& image) {
  return row >= 0 && col >= 0 && row < image.rows() && col < image.cols();
}

// Return the byte-size of an image.
template <class T>
uint64_t Size(const Image<T>& image) {
  return image.rows() * image.cols() * sizeof(T);
}
