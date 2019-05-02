
// Simple 2D vector library
#ifndef VEC_H_
#define VEC_H_

#include <stdbool.h>
#include <math.h>

typedef double vec_dimension;

// A two-dimensional vector.
struct Vec {
  vec_dimension x;  // The x-coordinate of the vector.
  vec_dimension y;  // The y-coordinate of the vector.
};

inline Vec Vec_make(const vec_dimension x, const vec_dimension y) {
  Vec vector;
  vector.x = x;
  vector.y = y;
  return vector;
}

inline Vec Vec_unit(double angle) {
  Vec vector;
  vector.x = cos(angle);
  vector.y = sin(angle);
  return vector;
}

// ************************* Fundamental attributes **************************

inline vec_dimension Vec_length(Vec vector) {
  return hypot(vector.x, vector.y);
}

inline double Vec_argument(Vec vector) {
  return atan2(vector.y, vector.x);
}

// ******************************* Arithmetic ********************************

inline bool Vec_equals(Vec lhs, Vec rhs) {
  return lhs.x == rhs.x && lhs.y == rhs.y;
}

inline Vec Vec_add(Vec lhs, Vec rhs) {
  return Vec_make(lhs.x + rhs.x, lhs.y + rhs.y);
}

inline Vec Vec_subtract(Vec lhs, Vec rhs) {
  return Vec_make(lhs.x - rhs.x, lhs.y - rhs.y);
}

inline Vec Vec_multiply(Vec vector, const double scalar) {
  return Vec_make(vector.x * scalar, vector.y * scalar);
}

inline Vec Vec_flip(Vec vector) {
  return Vec_make(-vector.x, -vector.y);
}

inline Vec Vec_divide(Vec vector, const double scalar) {
  return Vec_make(vector.x / scalar, vector.y / scalar);
}

inline vec_dimension Vec_dotProduct(Vec lhs, Vec rhs) {
  return lhs.x * rhs.x + lhs.y * rhs.y;
}

inline vec_dimension Vec_crossProduct(Vec lhs, Vec rhs) {
  return lhs.x * rhs.y - lhs.y * rhs.x;
}

inline Vec Vec_rotate(Vec v, double angle) {
  return Vec_make(v.x*cos(angle) - v.y*sin(angle), v.x*sin(angle) + v.y*cos(angle));
}

// **************************** Related vectors ******************************

inline Vec Vec_normalize(Vec vector) {
  return Vec_divide(vector, Vec_length(vector));
}

inline Vec Vec_orthogonal(Vec vector) {
  return Vec_make(-vector.y, vector.x);
}

// ******************** Relationships with other vectors *********************

inline double Vec_angle(Vec vector1, Vec vector2) {
  return Vec_argument(vector1) - Vec_argument(vector2);
}

inline vec_dimension Vec_component(Vec vector1, Vec vector2) {
  return Vec_length(vector1) * cos(Vec_angle(vector1, vector2));
}

inline Vec Vec_projectOnto(Vec vector1, Vec vector2) {
  return Vec_multiply(Vec_normalize(vector2), Vec_component(vector1, vector2));
}

#endif  // VEC_H_
