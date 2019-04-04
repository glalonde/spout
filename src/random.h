#pragma once
#include "base/logging.h"
#include "src/eigen_types.h"

// Get a uniform random floating point distribution
template <class Scalar, typename std::enable_if_t<
                            std::is_floating_point<Scalar>::value, int> = 0>
auto UniformRandomDistribution(const Scalar& min, const Scalar& max) {
  CHECK_LE(min, max);
  return std::uniform_real_distribution<Scalar>(min, max);
}

// Get a uniform random integer distribution
template <class Scalar,
          typename std::enable_if_t<std::is_integral<Scalar>::value, int> = 0>
auto UniformRandomDistribution(const Scalar& min, const Scalar& max) {
  CHECK_LE(min, max);
  return std::uniform_int_distribution<Scalar>(min, max);
}

// Initialize an eigen matrix-like object to a given distribution
template <class DistType, class GenType, class Derived>
void SetRandomDistribution(DistType* distribution, GenType* gen,
                           Eigen::DenseBase<Derived> const& vals) {
  auto& mutable_vals = const_cast<Eigen::DenseBase<Derived>&>(vals);
  mutable_vals = Eigen::DenseBase<Derived>::NullaryExpr(
      vals.rows(), vals.cols(), [&]() { return (*distribution)(*gen); });
}

// Initialize an eigen matrix-like object to a uniform distribution
template <class GenType, class Derived>
void SetRandomUniform(const typename Derived::Scalar& min,
                      const typename Derived::Scalar& max, GenType* gen,
                      Eigen::DenseBase<Derived> const& vals) {
  auto dist = UniformRandomDistribution(min, max);
  SetRandomDistribution(&dist, gen, vals);
}
