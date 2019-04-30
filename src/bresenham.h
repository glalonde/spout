#pragma once
#include <algorithm>
#include "src/eigen_types.h"
#include "src/image.h"
#include "src/int_grid.h"

static Eigen::Matrix<int, 4, 8> kToOctant0 =
    (Eigen::Matrix<int, 4, 8>() << 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, 1, 0, 0, -1,
     -1, 0, 0, 1, -1, 0, 0, -1, 1, 0, 1, 0, 0, 1, -1, 0, 0, -1)
        .finished();

static Eigen::Matrix<int, 4, 8> kFromOctant0 =
    (Eigen::Matrix<int, 4, 8>() << 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0, 0, -1,
     1, 0, 0, 1, 1, 0, 0, -1, -1, 0, 1, 0, 0, 1, -1, 0, 0, -1)
        .finished();

template<class IntType>
static Eigen::Matrix<IntType, 2, 8> kStepX =
    (Eigen::Matrix<IntType, 2, 8>() << 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, 1, 0, 0, -1,
     -1, 0)
        .finished();

template<class IntType>
static Eigen::Matrix<IntType, 2, 8> kStepY =
    (Eigen::Matrix<IntType, 2, 8>() << 0, 1, -1, 0, 0, -1, 1, 0, 1, 0, 0, 1, -1, 0,
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

/* >  >   >
 *  \2|1/
 *  3\|/0
 *^--------^
 *  4/|\7
 *  /5|6\
 * <  >  >
 */
template <class Scalar>
uint8_t GetOctant(const Vector2<Scalar>& vec) {
  if (vec.x() >= 0) {
    if (vec.y() >= 0) {
      if (vec.x() >= vec.y()) {
        return 0;
      } else {
        return 1;
      }
    } else if (vec.x() >= -vec.y()) {
      return 7;
    } else {
      return 6;
    }
  } else {
    if (vec.y() >= 0) {
      if (-vec.x() > vec.y()) {
        return 3;
      } else {
        return 2;
      }
    } else if (-vec.x() >= -vec.y()) {
      return 4;
    } else {
      return 5;
    }
  }
}

template <class T>
void SubPixelBresenhamNormal(const Vector2d& pos, const Vector2d& vel,
                             const double dt,
                             const T& buffer,
                             Vector2d* pos_out, Vector2d* vel_out) {
  using CellType = typename T::Scalar;
  uint8_t octant = GetOctant(vel);
  Vector2d pos_tf = TransformToOctant0(octant, pos);
  Vector2d vel_tf = TransformToOctant0(octant, vel);
  const double slope = vel_tf.y() / vel_tf.x();
  Vector2i pos_i = pos_tf.array().floor().matrix().cast<int>();
  Vector2d end_pos_tf = pos_tf + vel_tf * dt;
  Vector2i end_pos_i = end_pos_tf.array().floor().matrix().cast<int>();
  int cells = (end_pos_i - pos_i).lpNorm<1>();
  Vector2i x_step = kStepX<int>.col(octant);
  Vector2i y_step = kStepY<int>.col(octant);
  Vector2d start_remainder = pos_tf - pos_i.cast<double>();
  pos_i = pos.cast<int>();

  // End remainder is relative to the center of the final pixel.
  Vector2d end_remainder =
      end_pos_tf - (end_pos_i.cast<double>() + Vector2d(.5, .5));

  auto is_on_buffer = [&buffer](int row, int col) -> bool {
    return row >= 0 && col >= 0 && row < buffer.rows() && col < buffer.cols();
  };

  // Doesn't really help the problem, since it doesn't change the particle at
  // all.
  if (!is_on_buffer(pos_i.y(), pos_i.x())) {
    LOG(ERROR) << "Particle not on buffers";
    *pos_out = pos;
    *vel_out = vel;
    return;
  }

  // This the "y-error" entering the next column
  double error = (1.0 - start_remainder.x()) * slope + start_remainder.y() - 1;
  while (cells > 0) {
    if (error > 0) {
      // Exit this pixel via the top.
      pos_i += y_step;
      const bool off_buffer = !is_on_buffer(pos_i.y(), pos_i.x());
      --error;
      if (off_buffer || buffer(pos_i.y(), pos_i.x()) > CellType(0)) {
        pos_i -= y_step;
        y_step *= -1;
        octant = kOctantFlipOverX[octant];
      }
    } else {
      // Exit this pixel via the right.
      pos_i += x_step;
      const bool off_buffer = !is_on_buffer(pos_i.y(), pos_i.x());
      error += slope;
      if (off_buffer || buffer(pos_i.y(), pos_i.x()) > CellType(0)) {
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

template <class T>
void DestructingBresenham(const Vector2d& pos, const Vector2d& vel,
                          const double dt, const double damage_rate, T* buffer,
                          Vector2d* pos_out, Vector2d* vel_out) {
  using CellType = typename T::Scalar;
  uint8_t octant = GetOctant(vel);
  Vector2d pos_tf = TransformToOctant0(octant, pos);
  Vector2d vel_tf = TransformToOctant0(octant, vel);
  const double slope = vel_tf.y() / vel_tf.x();
  Vector2i pos_i = pos_tf.array().floor().matrix().cast<int>();
  Vector2d end_pos_tf = pos_tf + vel_tf * dt;
  Vector2i end_pos_i = end_pos_tf.array().floor().matrix().cast<int>();
  int cells = (end_pos_i - pos_i).lpNorm<1>();
  Vector2i x_step = kStepX<int>.col(octant);
  Vector2i y_step = kStepY<int>.col(octant);
  Vector2d start_remainder = pos_tf - pos_i.cast<double>();
  pos_i = pos.cast<int>();

  // End remainder is relative to the center of the final pixel.
  Vector2d end_remainder =
      end_pos_tf - (end_pos_i.cast<double>() + Vector2d(.5, .5));

  auto is_on_buffer = [&buffer](int row, int col) -> bool {
    return row >= 0 && col >= 0 && row < buffer->rows() && col < buffer->cols();
  };

  auto damage_cell = [&vel, damage_rate](CellType* cell) {
    const CellType damage_amount = std::clamp(
        static_cast<int>(damage_rate * vel.norm()), 0, static_cast<int>(*cell));
    *cell -= damage_amount;
  };

  // Doesn't really help the problem, since it doesn't change the particle at
  // all.
  if (!is_on_buffer(pos_i.y(), pos_i.x())) {
    *pos_out = pos;
    *vel_out = vel;
    return;
  }

  // This the "y-error" entering the next column
  double error = (1.0 - start_remainder.x()) * slope + start_remainder.y() - 1;
  while (cells > 0) {
    if (error > 0) {
      // Exit this pixel via the top.
      pos_i += y_step;
      --error;
      const bool off_buffer = !is_on_buffer(pos_i.y(), pos_i.x());
      if (off_buffer || (*buffer)(pos_i.y(), pos_i.x()) > CellType(0)) {
        if (!off_buffer) {
          damage_cell(&(*buffer)(pos_i.y(), pos_i.x()));
        }
        // Bounce
        pos_i -= y_step;
        y_step *= -1;
        octant = kOctantFlipOverX[octant];
      }
    } else {
      // Exit this pixel via the right.
      pos_i += x_step;
      error += slope;
      const bool off_buffer = !is_on_buffer(pos_i.y(), pos_i.x());
      if (off_buffer || (*buffer)(pos_i.y(), pos_i.x()) > CellType(0)) {
        if (!off_buffer) {
          damage_cell(&(*buffer)(pos_i.y(), pos_i.x()));
        }
        // Bounce
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

template <class T>
void BresenhamExperiment(const Vector2i& pos, const Vector2i& vel,
                         const int cell_size /* fixed point resolution */,
                         const double dt, const T& buffer, Vector2i* pos_out,
                         Vector2i* vel_out) {
  using CellType = typename T::Scalar;
  const Vector2i kHalfCell = Vector2i::Constant(cell_size / 2);
  // Signed integer division and floor
  auto floor_div = [](int a, int b) {
    int d = a / b;
    int r = a % b;
    return r ? (d - ((a < 0) ^ (b < 0))) : d;
  };

  auto get_cell = [cell_size, &floor_div](const Vector2i& vec) -> Vector2i {
    return vec.unaryExpr(
        [&cell_size, &floor_div](int v) { return floor_div(v, cell_size); });
  };

  // Hot spot
  Vector2i end_pos = pos + (vel.cast<double>() * dt).cast<int>();
  Vector2i delta = (end_pos - pos).cwiseAbs();
  Vector2i step(pos.x() < end_pos.x() ? 1 : -1, pos.y() < end_pos.y() ? 1 : -1);
  // Hot spot
  Vector2i pos_i = get_cell(pos);
  // Hot spot
  Vector2i end_pos_i = get_cell(end_pos);
  Vector2i delta_i = end_pos_i - pos_i;
  Vector2i start_remainder = pos_i * cell_size + kHalfCell - pos;
  start_remainder = start_remainder.cwiseProduct(step);
  *vel_out = vel;

  int error = delta.x() * start_remainder.y() - delta.y() * start_remainder.x();
  Vector2i end_remainder = end_pos - end_pos_i * cell_size;

  auto is_on_buffer = [&buffer](int row, int col) -> bool {
    auto ans =
        row >= 0 && col >= 0 && row < buffer.rows() && col < buffer.cols();
    return ans;
  };

  if (!is_on_buffer(pos_i.y(), pos_i.x())) {
    *pos_out = pos;
    *vel_out = vel;
    return;
  }

  int num_cells = delta_i.lpNorm<1>();
  delta *= cell_size;

  while (num_cells > 0) {
    const int error_horizontal = error - delta.y();
    const int error_vertical = error + delta.x();
    if (error_vertical > -error_horizontal) {
      // Horizontal step
      error = error_horizontal;
      pos_i.x() += step.x();
      // Bounce horizontally
      const bool off_buffer = !is_on_buffer(pos_i.y(), pos_i.x());
      if (off_buffer || buffer(pos_i.y(), pos_i.x()) > CellType(0)) {
        pos_i.x() -= step.x();
        step.x() *= -1;
        vel_out->x() *= -1;
        end_remainder.y() = cell_size - end_remainder.y();
      }
    } else {
      // Vertical step
      error = error_vertical;
      pos_i.y() += step.y();

      // Bounce vertically
      const bool off_buffer = !is_on_buffer(pos_i.y(), pos_i.x());
      if (off_buffer || buffer(pos_i.y(), pos_i.x()) > CellType(0)) {
        pos_i.y() -= step.y();
        end_remainder.x() = cell_size - end_remainder.x();
        step.y() *= -1;
        vel_out->y() *= -1;
      }
    }
    --num_cells;
  }
  // Hot spot
  *pos_out = pos_i * cell_size + end_remainder;
}

template <class T>
void BresenhamExperimentLowRes(const Vector2u32& pos, const Vector2i& vel,
                               const double dt, const T& buffer,
                               Vector2u32* pos_out, Vector2i* vel_out) {
  using CellType = typename T::Scalar;

  auto get_cell = [](const Vector2u32& vec) -> Vector2i {
    return vec.unaryExpr([](uint32_t v) -> int {
      return static_cast<int>(GetLowRes<8>(v)) - kAnchor<uint32_t, 8>;
    });
  };

  auto get_remainder = [](const Vector2u32& vec) -> Vector2u32 {
    return vec.unaryExpr(
        [](uint32_t v) -> uint32_t { return GetHighRes<8>(v); });
  };

  Vector2i signed_delta = (vel.cast<double>() * dt).cast<int>();

  Vector2u32 end_pos(static_cast<int>(pos.x()) + signed_delta.x(),
                     static_cast<int>(pos.y()) + signed_delta.y());

  Vector2i delta = signed_delta.cwiseAbs();
  Vector2i step(signed_delta.x() > 0 ? 1 : -1, signed_delta.y() > 0 ? 1 : -1);
  Vector2i pos_i = get_cell(pos);
  Vector2i end_pos_i = get_cell(end_pos);
  Vector2i delta_i = end_pos_i - pos_i;

  Vector2i start_remainder =
      Vector2u32::Constant(kHalfCell<uint32_t, 8>).template cast<int>() -
      get_remainder(pos).template cast<int>();
  start_remainder = start_remainder.cwiseProduct(step);
  *vel_out = vel;

  int error = delta.x() * start_remainder.y() - delta.y() * start_remainder.x();
  Vector2i end_remainder = get_remainder(end_pos).template cast<int>();

  auto is_on_buffer = [&buffer](int row, int col) -> bool {
    auto ans =
        row >= 0 && col >= 0 && row < buffer.rows() && col < buffer.cols();
    return ans;
  };

  if (!is_on_buffer(pos_i.y(), pos_i.x())) {
    *pos_out = pos;
    *vel_out = vel;
    return;
  }

  int num_cells = delta_i.lpNorm<1>();
  delta *= kCellSize<uint32_t, 8>;

  while (num_cells > 0) {
    const int error_horizontal = error - delta.y();
    const int error_vertical = error + delta.x();
    if (error_vertical > -error_horizontal) {
      // Horizontal step
      error = error_horizontal;
      pos_i.x() += step.x();
      // Bounce horizontally
      const bool off_buffer = !is_on_buffer(pos_i.y(), pos_i.x());
      if (off_buffer || buffer(pos_i.y(), pos_i.x()) > CellType(0)) {
        pos_i.x() -= step.x();
        step.x() *= -1;
        vel_out->x() *= -1;
        end_remainder.y() = kCellSize<uint32_t, 8> - end_remainder.y();
      }
    } else {
      // Vertical step
      error = error_vertical;
      pos_i.y() += step.y();

      // Bounce vertically
      const bool off_buffer = !is_on_buffer(pos_i.y(), pos_i.x());
      if (off_buffer || buffer(pos_i.y(), pos_i.x()) > CellType(0)) {
        pos_i.y() -= step.y();
        end_remainder.x() = kCellSize<uint32_t, 8> - end_remainder.x();
        step.y() *= -1;
        vel_out->y() *= -1;
      }
    }
    --num_cells;
  }

  CHECK_EQ(end_pos_i, pos_i);

  *pos_out = pos_i.unaryExpr([](int v) -> uint32_t {
    return SetLowRes<8>(static_cast<uint32_t>(v + kAnchor<uint32_t, 8>));
  });
  pos_out->x() += end_remainder.x();
  pos_out->y() += end_remainder.y();
}
