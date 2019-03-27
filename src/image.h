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
using RGBAF32 = Vector4<float>;
}  // namespace PixelType
