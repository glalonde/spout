#pragma once

#include "src/image.h"

namespace details {

template <class Derived>
Derived VectorScaleClamp(const Eigen::MatrixBase<Derived>& input,
                         typename Derived::Scalar factor,
                         typename Derived::Scalar min,
                         typename Derived::Scalar max) {
  return (input * factor).cwiseMin(max).cwiseMax(min);
}
}  // namespace details

template <class Output, class Input>
Output Convert(const Input& input);

template <>
PixelType::RGBU8 Convert<PixelType::RGBU8, PixelType::RGBAU8>(
    const PixelType::RGBAU8& input) {
  return input.head<3>(0);
}

template <>
PixelType::RGBAU8 Convert<PixelType::RGBAU8, PixelType::RGBU8>(
    const PixelType::RGBU8& input) {
  PixelType::RGBAU8 out;
  out.head<3>() = input;
  out[3] = 255;
  return out;
}

template <>
PixelType::RGBU8 Convert<PixelType::RGBU8, PixelType::RGBF32>(
    const PixelType::RGBF32& input) {
  return details::VectorScaleClamp(input, 256.0, 0.0, 255.0).cast<uint8_t>();
}

template <>
PixelType::RGBAU8 Convert<PixelType::RGBAU8, PixelType::RGBAF32>(
    const PixelType::RGBAF32& input) {
  return details::VectorScaleClamp(input, 256.0, 0.0, 255.0).cast<uint8_t>();
}

template <>
PixelType::RGBAU8 Convert<PixelType::RGBAU8, PixelType::RGBF32>(
    const PixelType::RGBF32& input) {
  return Convert<PixelType::RGBAU8, PixelType::RGBU8>(
      Convert<PixelType::RGBU8, PixelType::RGBF32>(input));
}

template <>
PixelType::RGBAU8 Convert<PixelType::RGBAU8, PixelType::RGBF64>(
    const PixelType::RGBF64& input) {
  return Convert<PixelType::RGBAU8, PixelType::RGBU8>(
      Convert<PixelType::RGBU8, PixelType::RGBF64>(input));
}
