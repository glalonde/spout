#pragma once
#include "src/eigen_types.h"

template <class T>
Matrix4<T> Perspective(T fovy /* radians */, T aspect, T z_near, T z_far) {
  DCHECK_GT(aspect, 0);
  DCHECK_GT(z_far, z_near);

  const double tan_half_fovy = std::tan(fovy / 2.0);
  Matrix4<T> res = Matrix4<T>::Zero();
  res(0, 0) = 1.0 / (aspect * tan_half_fovy);
  res(1, 1) = 1.0 / (tan_half_fovy);
  res(2, 2) = -(z_far + z_near) / (z_far - z_near);
  res(3, 2) = -1.0;
  res(2, 3) = -(2.0 * z_far * z_near) / (z_far - z_near);
  return res;
}

template <class T>
Matrix4<T> LookAt(const Vector3<T>& eye, const Vector3<T>& center,
                  const Vector3<T>& up) {
  Vector3<T> f = (center - eye).normalized();
  Vector3<T> u = up.normalized();
  Vector3<T> s = f.cross(u).normalized();
  u = s.cross(f);

  Matrix4<T> res;
  res << s.x(), s.y(), s.z(), -s.dot(eye), u.x(), u.y(), u.z(), -u.dot(eye),
      -f.x(), -f.y(), -f.z(), f.dot(eye), 0, 0, 0, 1;
  return res;
}
