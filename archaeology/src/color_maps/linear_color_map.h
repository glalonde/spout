#pragma once

#include "base/logging.h"
#include "src/eigen_types.h"
#include "src/interpolation.h"

template <class Derived>
Vector3f EvaluateLinearColorMap(double p,
                                const Eigen::MatrixBase<Derived>& mat) {
  p *= (Derived::ColsAtCompileTime - 1);
  int lower = std::floor(p);
  if (lower >= Derived::ColsAtCompileTime - 1) {
    return mat.col(Derived::ColsAtCompileTime - 1);
  } else if (lower < 0) {
    return mat.col(0);
  } else {
    const Vector3f& color1 = mat.col(lower);
    const Vector3f& color2 = mat.col(lower + 1);
    auto out = InterpolateLinear(p - lower, color1, color2);
    return out;
  }
}
