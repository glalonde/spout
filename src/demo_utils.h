#pragma once

#include "src/convert.h"
#include "src/fonts/font_renderer.h"
#include "src/image.h"
#include "src/random.h"
#include "src/buffer_stack.h"

static constexpr uint8_t kWall = std::numeric_limits<uint8_t>::max();
static const PixelType::RGBAU8 kWallColor = {0, 0, 255, 255};
static const PixelType::RGBAU8 kParticleColor = {0, 255, 0, 255};
static const PixelType::RGBAU8 kShipColor = {255, 255, 0, 255};
static const PixelType::RGBAU8 kTrailColor = {0, 128, 0, 255};
static const PixelType::RGBAU8 text_color =
    Convert<PixelType::RGBAU8>(PixelType::RGBF64(1.0, 0.0, 0.0));

void RenderEnvironment(const Image<uint8_t>& env,
                       Image<PixelType::RGBAU8>* data) {
  for (int r = 0; r < env.rows(); ++r) {
    for (int c = 0; c < env.cols(); ++c) {
      if (env(r, c) == kWall) {
        (*data)(r, c) = kWallColor;
      } else {
        (*data)(r, c) = {0, 0, 0, 255};
      }
    }
  }
}

template <class T>
void AddSideWalls(const T& wall_value, Image<T>* data) {
  // Set left and right to walls
  for (int r = 0; r < data->rows(); ++r) {
    (*data)(r, 0) = wall_value;
    (*data)(r, data->cols() - 1) = wall_value;
  }
}

template <class T>
void AddBottomWall(const T& wall_value, Image<T>* data) {
  // Set top and bottom to walls
  for (int c = 0; c < data->cols(); ++c) {
    (*data)(0, c) = wall_value;
  }
}

template <class T>
void AddTopWall(const T& wall_value, Image<T>* data) {
  for (int c = 0; c < data->cols(); ++c) {
    (*data)(data->rows() - 1, c) = wall_value;
  }
}

template <class T>
void AddAllWalls(const T& wall_value, Image<T>* data) {
  AddTopWall(wall_value, data);
  AddBottomWall(wall_value, data);
  AddSideWalls(wall_value, data);
}

void AddFpsText(double fps, const PixelType::RGBAU8& color,
                Image<PixelType::RGBAU8>* data) {
  std::string fps_string = FormatString("%.0f", fps);
  RenderString(fps_string, {1, 1}, color, 1,
               font_rendering::Justification::kLeft, data);
}

template <class T>
void AddNoise(const T& wall_value, double percent_filled, std::mt19937* gen,
              Image<T>* data) {
  Image<double> perlin_vals(data->rows(), data->cols());
  PerlinNoise(0.0, 1.0, data->cols() / 10, gen, perlin_vals);
  (*data) = perlin_vals.unaryExpr(
      [percent_filled, wall_value](double noise_val) -> T {
        if (noise_val <= percent_filled) {
          return wall_value;
        } else {
          return T(0);
        }
      });
}

// Returns the min corner (row, col) and the (rows, cols) sizes of the smallest
// concentric ring that is nested inside the given dimensions `rows_cols`
void SmallestConcentricRing(const Vector2i& rows_cols, Vector2i* min_corner,
                            Vector2i* sizes) {
  const auto& rows = rows_cols[0];
  const auto& cols = rows_cols[1];
  if (rows <= cols) {
    (*sizes)[0] = rows % 2;
    (*sizes)[1] = cols - rows + (*sizes)[0];
  } else {
    (*sizes)[1] = cols % 2;
    (*sizes)[0] = rows - cols + (*sizes)[1];
  }
  (*min_corner) = rows_cols / 2 - (*sizes) / 2;
}

// Returns the coordinates of the first nonzero cell spiraling out of the
// center, or nullopt if the whole thing is full.
std::optional<Vector2i> FindEmptySpot(const Image<uint8_t>& env) {
  Vector2i sizes;
  Vector2i min_corner;
  Vector2i rows_cols(env.rows(), env.cols());
  SmallestConcentricRing(rows_cols, &min_corner, &sizes);
  while (sizes[0] <= env.rows() && sizes[1] <= env.cols()) {
    Vector2i max_corner = min_corner.array() + (sizes.array() - 1);
    // min and max cols
    for (int r = min_corner[0]; r <= max_corner[0]; ++r) {
      if (env(r, min_corner[1]) <= 0) {
        return Vector2i(r, min_corner[1]);
      } else if (env(r, max_corner[1]) <= 0) {
        return Vector2i(r, max_corner[1]);
      }
    }
    // min and max rows
    for (int c = min_corner[1]; c <= max_corner[1]; ++c) {
      if (env(min_corner[0], c) <= 0) {
        return Vector2i(min_corner[0], c);
      } else if (env(max_corner[0], c) <= 0) {
        return Vector2i(max_corner[0], c);
      }
    }
    sizes.array() += 2;
    min_corner.array() -= 1;
  }
  return {};
}
