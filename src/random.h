#pragma once
#include "base/logging.h"
#include "src/eigen_types.h"
#include "src/interpolation.h"

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

// Returns noise values from [-1, 1]
template <class GenType, class Derived>
void PerlinNoise(typename Derived::Scalar min_val,
                 typename Derived::Scalar max_val, int cell_size,
                 GenType* rand_gen, Eigen::DenseBase<Derived> const& vals) {
  // Coarse grained sampling grid to set the "frequency" of the noise.
  const int grid_rows =
      static_cast<int>(std::ceil(vals.rows() / cell_size)) + 1;
  const int grid_cols =
      static_cast<int>(std::ceil(vals.cols() / cell_size)) + 1;

  // Initialize random gradient vectors
  using VectorMat = Eigen::Array<Vector2d, Eigen::Dynamic, Eigen::Dynamic>;
  auto dist = UniformRandomDistribution(-M_PI, M_PI);
  auto gradient_gen = [&dist, rand_gen]() {
    const double angle = dist(*rand_gen);
    return Vector2d(std::cos(angle), std::sin(angle));
  };
  VectorMat gradients =
      VectorMat::NullaryExpr(grid_rows, grid_cols, gradient_gen);

  // Dot product of delta vector and gradient vector
  auto dot_grid_gradient = [&gradients](int row, int col,
                                        const Vector2d& point) -> double {
    const Vector2d delta = point - Vector2d(col, row);
    return gradients(row, col).cwiseProduct(delta).sum();
  };

  // Compute bilinear interpolation at a given row/col in the fine grain
  // sampling grid.
  auto sample_location = [&](int row, int col) {
    Vector2d sample_point(col + .5, row + .5);
    sample_point /= cell_size;
    const int g_row0 = static_cast<int>(sample_point[1]);
    const int g_row1 = g_row0 + 1;
    const int g_col0 = static_cast<int>(sample_point[0]);
    const int g_col1 = g_col0 + 1;

    const Vector2d delta = sample_point - Vector2d(g_col0, g_row0);

    // Gradient wrt change in row(y) on the left edge
    const double n0 = dot_grid_gradient(g_row0, g_col0, sample_point);
    const double n1 = dot_grid_gradient(g_row1, g_col0, sample_point);
    const double v0 = InterpolateLinear(delta.y(), n0, n1);

    // Gradient wrt change in row(y) on the right edge
    const double n2 = dot_grid_gradient(g_row0, g_col1, sample_point);
    const double n3 = dot_grid_gradient(g_row1, g_col1, sample_point);
    const double v1 = InterpolateLinear(delta.y(), n2, n3);

    return InterpolateLinear(delta.x(), v0, v1);
  };

  auto& mutable_vals = const_cast<Eigen::DenseBase<Derived>&>(vals);
  for (int r = 0; r < vals.rows(); ++r) {
    for (int c = 0; c < vals.cols(); ++c) {
      mutable_vals(r, c) = sample_location(r, c);
    }
  }

  CHECK_LE(min_val, max_val);
  // Scale and shift to match the desired range of [min_val, max_val]
  //
  // See: http://digitalfreepen.com/2017/06/20/range-perlin-noise.html
  static const typename Derived::Scalar native_range(2.0 * std::sqrt(.5));
  const auto range_multiplier = (max_val - min_val) / native_range;
  mutable_vals *= range_multiplier;
  const auto shift = min_val - (-range_multiplier * std::sqrt(.5));

  // TODO: Better way to add a scalar to a dense base???
  mutable_vals = mutable_vals.unaryExpr(
      [&shift](typename Derived::Scalar v) { return v + shift; });
}
