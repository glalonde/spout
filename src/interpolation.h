#pragma once

template <class Scalar, class Value>
Value InterpolateLinear(const Scalar& p, const Value& v0, const Value& v1) {
  return (Scalar(1.0) - p) * v0 + p * v1;
}
