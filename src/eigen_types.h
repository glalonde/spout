#pragma once
#include <stdint.h>
#include <Eigen/Core>
#include <Eigen/Geometry>

// N-vector using Scalar
template <class Scalar, int N>
using Vector = Eigen::Matrix<Scalar, N, 1>;

// 1-vector using Scalar
template <class Scalar>
using Vector1 = Vector<Scalar, 1>;

using Vector1f = Vector1<float>;
using Vector1d = Vector1<double>;
using Vector1u16 = Vector1<uint16_t>;

// 2-vector using Scalar
template <class Scalar>
using Vector2 = Vector<Scalar, 2>;

using Eigen::Vector2d;
using Eigen::Vector2f;
using Eigen::Vector2i;
using Vector2u16 = Vector2<uint16_t>;

// 3-vector using Scalar
template <class Scalar>
using Vector3 = Vector<Scalar, 3>;

using Eigen::Vector3d;
using Eigen::Vector3f;
using Eigen::Vector3i;
using Vector3u16 = Vector3<uint16_t>;

// 4-vector using Scalar
template <class Scalar>
using Vector4 = Vector<Scalar, 4>;
using Vector4u8 = Vector4<uint8_t>;
using Vector4u16 = Vector4<uint16_t>;
using Vector4u32 = Vector4<uint32_t>;
using Vector4i = Vector4<int32_t>;
using Eigen::Vector4d;
using Eigen::Vector4f;

// 6-vector using Scalar
template <class Scalar>
using Vector6 = Vector<Scalar, 6>;

using Vector6d = Vector6<double>;
using Vector6f = Vector6<float>;
using Vector6b = Vector6<bool>;

// 7-vector using Scalar
template <class Scalar>
using Vector7 = Vector<Scalar, 7>;

using Vector7d = Vector7<double>;
using Vector7f = Vector7<float>;

// 8-vector using Scalar
template <class Scalar>
using Vector8 = Vector<Scalar, 8>;

using Vector8d = Vector8<double>;
using Vector8f = Vector8<float>;
using Vector8b = Vector8<bool>;
using Vector8u16 = Vector8<uint16_t>;

// N-vector (runtime) using Scalar
template <typename Scalar>
using VectorX = Eigen::Matrix<Scalar, Eigen::Dynamic, 1>;
using VectorXd = VectorX<double>;
using VectorXi = VectorX<int32_t>;
using VectorXf = VectorX<float>;
using VectorXb = VectorX<bool>;

using Eigen::Matrix;

// (2x2) matrix using Scalar
template <class Scalar>
using Matrix2 = Matrix<Scalar, 2, 2>;

using Eigen::Matrix2d;
using Eigen::Matrix2f;
using Eigen::Matrix2Xd;
using Eigen::Matrix2Xf;
using Eigen::MatrixX2d;
using Eigen::MatrixX2f;

// (3x3) matrix using Scalar
template <class Scalar>
using Matrix3 = Matrix<Scalar, 3, 3>;

using Eigen::Matrix3d;
using Eigen::Matrix3f;
using Eigen::Matrix3Xd;
using Eigen::Matrix3Xf;
using Eigen::MatrixX3d;
using Eigen::MatrixX3f;

// (4x4) matrix using Scalar
template <class Scalar>
using Matrix4 = Matrix<Scalar, 4, 4>;

using Eigen::Matrix4d;
using Eigen::Matrix4f;
using Eigen::Matrix4Xd;
using Eigen::Matrix4Xf;
using Eigen::MatrixX4d;
using Eigen::MatrixX4f;

// 6 x 6 matrix using Scalar
template <class Scalar>
using Matrix6 = Matrix<Scalar, 6, 6>;

using Matrix6f = Matrix6<float>;

using Matrix6d = Matrix6<double>;
using Matrix6Xf = Matrix<float, 6, Eigen::Dynamic>;
using Matrix6Xd = Matrix<double, 6, Eigen::Dynamic>;
using MatrixX6f = Matrix<float, Eigen::Dynamic, 6>;
using MatrixX6d = Matrix<double, Eigen::Dynamic, 6>;

// 6 x 6 matrix using Scalar
template <class Scalar>
using MatrixX = Matrix<Scalar, Eigen::Dynamic, Eigen::Dynamic>;
using MatrixXf = MatrixX<float>;
using MatrixXd = MatrixX<double>;

template <class Scalar, int Dim>
using AlignedBox = Eigen::AlignedBox<Scalar, Dim>;
template <class Scalar>
using AlignedBox2 = AlignedBox<Scalar, 2>;
using AlignedBox2d = AlignedBox2<double>;
using AlignedBox2f = AlignedBox2<float>;

// Set the values in the matrix to random values between min and max
template <class Derived>
void SetRandomRange(typename Derived::Scalar min, typename Derived::Scalar max,
                    const Eigen::MatrixBase<Derived>& vals) {
  auto& mutable_vals = const_cast<Eigen::MatrixBase<Derived>&>(vals);
  mutable_vals.setRandom();
  auto half_range = (max - min) / typename Derived::Scalar(2.0);
  mutable_vals = mutable_vals * half_range;
  auto mid_point = (max + min) / typename Derived::Scalar(2.0);
  mutable_vals = mutable_vals.array() + mid_point;
}
