#pragma once
#include "src/eigen_types.h"

// Barebones SO2 group aka. 2D planar rotations.
// Basically a complex number, normalized -> point on unit circle.
template<class T>
struct SO2 {
 public:
  SO2() : SO2(T(1), T(0)) {}
  SO2(T radians) : SO2(std::cos(radians), std::sin(radians)) {}
  SO2(T cos, T sin) : data_(cos, sin) {}
  SO2(const SO2& other) = default;

  // Compose two rotations with multiplication
  template <class U>
  SO2<T>& operator*=(const SO2<U>& z) {
    const T new_cos = cos() * z.cos() - sin() * z.sin();
    data_[1] = cos() * z.sin() + sin() * z.cos();
    data_[0] = new_cos;
    const T squared_norm = data_.squaredNorm();
    if (squared_norm != T(1.0)) {
      const T scale = T(2.0) / (T(1.0) + squared_norm);
      data_ *= scale;
    }
    return *this;
  }

  SO2<T> inverse() const {
    return SO2(cos(), -sin());
  }

  const Vector2<T>& data() const {
    return data_;
  }

  const T& cos() const {
    return data_.x();
  }

  const T& sin() const {
    return data_.y();
  }

  void Normalize() {
    DCHECK_GT(data_.squaredNorm(), std::numeric_limits<T>::epsilon());
    data_.normalize();
  }

  T radians() const {
    return std::atan2(sin(), cos());
  }

 private:
  Vector2<T> data_;
};

template <class T>
inline SO2<T> operator*(const SO2<T>& x, const SO2<T>& y) {
  SO2<T> r = x;
  r *= y;
  return r;
}

using SO2f = SO2<float>;
using SO2d = SO2<double>;
