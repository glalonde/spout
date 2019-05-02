
#ifndef TILE_BUFFER_H_
#define TILE_BUFFER_H_

#include <iostream>
#include "constants.h"
#include "types.h"
#include "screen.h"
#include "color_utils.h"
#include "packed_grid/packed_grid_64.h"

static const int NUM_TYPES = 255;

class TileBuffer {
public:
  static const int width = LEVEL_WIDTH;
  static const int height = LEVEL_HEIGHT;
  int id;

  TileBuffer();
  TileBuffer(pixel_t start_color, pixel_t end_color);
  void Reset();
  void SetID(int id);
  void Draw(Screen<counter_t>* screen, int origin_y);
  void Clear();
  void SetAll(cell_t type);
  void DrawLine(int x, int y, int x1, int y1, cell_t type);
  void FillRect(int x, int y, int width, int height, cell_t type);
  void CoolRect(int left, int bot, int width, int height, int n_lines, cell_t type);

  // Setters
  void SetCellDirect(int x, int y, cell_t type);
  void SetCell(int x, int y, cell_t type);

  // Accessors
  cell_t GetCellDirect(int x, int y);
  cell_t GetCell(int x, int y);
  bool GetOccupancyDirect(int x, int y);
  bool GetOccupancy(int x, int y);
  block_t GetBlock(int x, int y);
  Collision CheckPair(IntVec curr, IntVec end);

  // Determine whether the given coordinates are off of this buffer
  // as determined by it's size.
  bool IsOffX(int x);
  bool IsOffY(int y);
  bool IsOff(int x, int y);
  void PrintOcc();
  
protected:
  // The main cell type storage structure.
  cell_t cells[LEVEL_HEIGHT][LEVEL_WIDTH];
  int buffer_size;

  // Packed occupancy grid.
  PackedGrid occupancy;

  // Color map from integer types to colors
  pixel_t color_map[NUM_TYPES];
};

inline cell_t TileBuffer::GetCellDirect(int x, int y) {
  return cells[y][x];
}

inline cell_t TileBuffer::GetCell(int x, int y) {
  if (!IsOff(x, y)) {
    return GetCellDirect(x, y);
  }
  return cell_types::TERRAIN;
}

inline bool TileBuffer::GetOccupancyDirect(int x, int y) {
  return occupancy.GetCell(x, y);
}

inline bool TileBuffer::GetOccupancy(int x, int y) {
  if (!IsOff(x, y)) {
    return GetOccupancyDirect(x, y);
  }
  return true;
}

inline block_t TileBuffer::GetBlock(int x, int y) {
  return occupancy.GetBlock(x, y);
}

inline Collision TileBuffer::CheckPair(IntVec curr, IntVec end) {
  return occupancy.CheckPair(curr, end);
}

inline void TileBuffer::SetCellDirect(int x, int y, cell_t type) {
  assert(x >= 0 && x < width);
  assert(y >= 0 && y < height);
  cells[y][x] = type;
  if (type == cell_types::EMPTY) {
    occupancy.ClearCell(x, y);
  } else {
    occupancy.SetCell(x, y);
  }
}

inline void TileBuffer::SetCell(int x, int y, cell_t type) {
  if (!IsOff(x, y)) {
    SetCellDirect(x, y, type);
  }
}

inline bool TileBuffer::IsOffX(int x) {
  return x < 0 || x >= this->width;
}

inline bool TileBuffer::IsOffY(int y) {
  return y < 0 || y >= this->height;
}

inline bool TileBuffer::IsOff(int x, int y) {
  return IsOffX(x) || (IsOffY(y));
}

inline void TileBuffer::SetID(int id) {
  this->id = id;
}

inline void TileBuffer::PrintOcc() {
  printf("++++++++++++++++++\n");
  for (int i = height - 1; i >= 0; i--) {
    for (int j = 0; j < width; j++) {
      if (occupancy.GetCell(j, i)) {
        printf("#");
      } else {
        printf(" ");
      }
    }
    printf("\n");
  }
  printf("++++++++++++++++++\n");
}

#endif
