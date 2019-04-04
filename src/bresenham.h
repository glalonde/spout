#pragma once
#include "src/eigen_types.h"
#include "src/image.h"

static Eigen::Matrix<int, 4, 8> kToOctant0 =
    (Eigen::Matrix<int, 4, 8>() << 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, 1, 0, 0, -1,
     -1, 0, 0, 1, -1, 0, 0, -1, 1, 0, 1, 0, 0, 1, -1, 0, 0, -1)
        .finished();

static Eigen::Matrix<int, 4, 8> kFromOctant0 =
    (Eigen::Matrix<int, 4, 8>() << 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0, 0, -1,
     1, 0, 0, 1, 1, 0, 0, -1, -1, 0, 1, 0, 0, 1, -1, 0, 0, -1)
        .finished();

static Eigen::Matrix<int, 2, 8> kStepX =
    (Eigen::Matrix<int, 2, 8>() << 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, 1, 0, 0, -1,
     -1, 0)
        .finished();

static Eigen::Matrix<int, 2, 8> kStepY =
    (Eigen::Matrix<int, 2, 8>() << 0, 1, -1, 0, 0, -1, 1, 0, 1, 0, 0, 1, -1, 0,
     0, -1)
        .finished();

static Vector<uint8_t, 8> kOctantFlipOverX =
    (Vector<uint8_t, 8>() << 7, 2, 1, 4, 3, 6, 5, 0).finished();
static Vector<uint8_t, 8> kOctantFlipOverY =
    (Vector<uint8_t, 8>() << 3, 6, 5, 0, 7, 2, 1, 4).finished();

template <class Scalar>
Vector2<Scalar> TransformToOctant0(uint8_t octant, const Vector2<Scalar>& vec) {
  Vector2<Scalar> vec_out;
  const auto& tform = kToOctant0.col(octant);
  vec_out.x() = vec.x() * tform[0] + vec.y() * tform[1];
  vec_out.y() = vec.x() * tform[2] + vec.y() * tform[3];
  return vec_out;
}

template <class Scalar>
Vector2<Scalar> TransformFromOctant0(uint8_t octant,
                                     const Vector2<Scalar>& vec) {
  Vector2<Scalar> vec_out;
  const auto& tform = kFromOctant0.col(octant);
  vec_out.x() = vec.x() * tform[0] + vec.y() * tform[1];
  vec_out.y() = vec.x() * tform[2] + vec.y() * tform[3];
  return vec_out;
}

template <class Scalar>
uint8_t GetOctant(const Vector2<Scalar>& vec) {
  if (vec.x() > 0) {
    if (vec.y() > 0) {
      if (vec.x() > vec.y()) {
        return 0;
      } else {
        return 1;
      }
    } else if (vec.x() > -vec.y()) {
      return 7;
    } else {
      return 6;
    }
  } else {
    if (vec.y() > 0) {
      if (-vec.x() > vec.y()) {
        return 3;
      } else {
        return 2;
      }
    } else if (-vec.x() > -vec.y()) {
      return 4;
    } else {
      return 5;
    }
  }
}

template <class T>
void SubPixelBresenhamNormal(const Vector2d& pos, const Vector2d& vel,
                             const double dt, Image<T>* buffer,
                             Vector2d* pos_out, Vector2d* vel_out) {
  uint8_t octant = GetOctant(vel);

  Vector2d pos_tf = TransformToOctant0(octant, pos);
  Vector2d vel_tf = TransformToOctant0(octant, vel);
  const double slope = vel_tf.y() / vel_tf.x();
  Vector2i pos_i = pos_tf.array().floor().matrix().cast<int>();
  Vector2d end_pos_tf = pos_tf + vel_tf * dt;
  Vector2i end_pos_i = end_pos_tf.array().floor().matrix().cast<int>();
  int cells = (end_pos_i - pos_i).lpNorm<1>();
  Vector2i x_step = kStepX.col(octant);
  Vector2i y_step = kStepY.col(octant);
  Vector2d start_remainder = pos_tf - pos_i.cast<double>();
  pos_i = pos.cast<int>();

  // End remainder is relative to the center of the final pixel.
  Vector2d end_remainder =
      end_pos_tf - (end_pos_i.cast<double>() + Vector2d(.5, .5));

  // This the "y-error" entering the next column
  double error = (1.0 - start_remainder.x()) * slope + start_remainder.y() - 1;
  while (cells > 0) {
    if (error > 0) {
      // Exit this pixel via the top.
      pos_i += y_step;
      --error;
      if ((*buffer)(pos_i.y(), pos_i.x()) > T(0)) {
        pos_i -= y_step;
        y_step *= -1;
        octant = kOctantFlipOverX[octant];
      }
    } else {
      // Exit this pixel via the right.
      pos_i += x_step;
      error += slope;
      if ((*buffer)(pos_i.y(), pos_i.x()) > T(0)) {
        pos_i -= x_step;
        x_step *= -1;
        octant = kOctantFlipOverY[octant];
      }
    }
    --cells;
  }
  end_remainder = TransformFromOctant0(octant, end_remainder);
  *pos_out = Vector2d(.5, .5) + pos_i.cast<double>() + end_remainder;
  *vel_out = TransformFromOctant0(octant, vel_tf);
}