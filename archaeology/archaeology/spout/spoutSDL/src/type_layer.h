#ifndef TYPE_LAYER_H_
#define TYPE_LAYER_H_

#include <vector>
#include <iostream>
#include <assert.h>
#include "constants.h"
#include "types.h"
#include "mobile_object.h"
#include "vec.h"
#include "screen.h"
#include "tile_buffer.h"
#include "packed_grid/packed_grid_64.h"


typedef struct BlockY {
  int16_t level;
  int16_t y;
} BlockY;

class TypeLayer {
public:
  TileBuffer buff_1;
  TileBuffer buff_2;

  // Pointers to buff1 or buff2 depending. Alternate which one is on top/bot
  TileBuffer* bot_buff;
  TileBuffer* top_buff;
  
  int height;
  int width;

  // The global height of the bottom line of the bottom buffer
  int bottom_height;
  // First row of top buffer.
  int middle_height;
  // Row above the last row of the top buffer.
  int top_height;

  // Public functions
  TypeLayer();
  TypeLayer(pixel_t start_color, pixel_t end_color);
  virtual void Reset();
  
  bool SyncHeight(int ship_height);
  cell_t GetCell(int x, int y);
  void UpdateState(double time);
  void SetCell(int x, int y, cell_t type);
  bool DecreaseCell(int x, int y, int amount, int limit);
  void Draw(Screen<counter_t>* screen);
  int GetBuffCount(int y);

protected:
  cell_t GetCellInternal(int x, int y);
  cell_t GetCellDirect(int x, int y);
  bool GetOccupancyDirect(int x, int y);
  BlockY GetBlockY(int y);
  block_t GetBlock(int x, BlockY y);

  void SetCellInternal(int x, int y, cell_t type);
  bool IsOffBuffers(int x, int y);
  bool IsOffBuffersX(int x);
  bool IsOffBuffersY(int y);
  bool IsOffBuffersAbsoluteY(int y);
  bool IsOnTopBuffer(int y);
  bool IsOnBotBuffer(int y);
  virtual void SwapBuffers();
  virtual void UnSwapBuffers();
};

inline bool TypeLayer::IsOffBuffers(int x, int y) {
  return IsOffBuffersX(x) || (IsOffBuffersY(y));
}

inline bool TypeLayer::IsOffBuffersX(int x) {
  return x < 0 || x >= GRID_WIDTH;
}

inline bool TypeLayer::IsOffBuffersY(int y) {
  return y < 0 || y >= 2*LEVEL_HEIGHT;
}

inline bool TypeLayer::IsOffBuffersAbsoluteY(int y) {
  return y < this->bottom_height || y >= this->top_height;
}

inline bool TypeLayer::IsOnTopBuffer(int y) {
  return y >= this->middle_height && y < this->top_height;
}

inline bool TypeLayer::IsOnBotBuffer(int y) {
  return y >= this->bottom_height && y < this->middle_height;
}

inline cell_t TypeLayer::GetCell(int x, int y) {
  // Convert the absolute height into buffer relative height
  y -= bottom_height;
  
  return GetCellInternal(x, y);
}

inline cell_t TypeLayer::GetCellInternal(int x, int y) {
  // Make sure it isn't off in the y direction
  if (!IsOffBuffersY(y)) {
    // If it is off in the x direction, like hitting the wall, it "collides"
    if (IsOffBuffersX(x)) {
      return 0;
    }
    
    return GetCellDirect(x, y);
  }
  return 0;
}

inline cell_t TypeLayer::GetCellDirect(int x, int y) {
  // Determine which buffer it is on and return that
  if (y < bot_buff->height) {
    return bot_buff->GetCellDirect(x, y);
  } else {
    return top_buff->GetCellDirect(x, y - bot_buff->height);
  }
}

inline bool TypeLayer::GetOccupancyDirect(int x, int y) {
  // Determine which buffer it is on and return that
  if (y < bot_buff->height) {
    return bot_buff->GetOccupancyDirect(x, y);
  } else {
    return top_buff->GetOccupancyDirect(x, y - bot_buff->height);
  }
}


inline BlockY TypeLayer::GetBlockY(int y) {
  if (y < bot_buff->height) {
    assert((y / PackedGrid::BLOCK_HEIGHT) < (1 << 15));
    return {.level = (int16_t)bot_buff->id, .y = (int16_t)((y) / PackedGrid::BLOCK_HEIGHT)};
  } else {
    assert((y - bot_buff->height) / PackedGrid::BLOCK_HEIGHT < (1 << 15));
    return {.level = (int16_t)top_buff->id, .y = (int16_t)((y - bot_buff->height) / PackedGrid::BLOCK_HEIGHT)};
  }
}

inline block_t TypeLayer::GetBlock(int x, BlockY y) {
  assert(y.y < PackedGrid::BLOCKS_HIGH);
  assert(y.y >= 0);
  if (y.level == bot_buff->id) {
    return bot_buff->GetBlock(x, y.y);
  } else if (y.level == top_buff->id) {
    return top_buff->GetBlock(x, y.y);
  } else {
    assert(false);
    return (block_t)0;
  }
}

inline void TypeLayer::SetCell(int x, int y, cell_t type) {
  // Convert the absolute height into buffer relative height
  y -= bottom_height;
  SetCellInternal(x, y, type);
}

inline void TypeLayer::SetCellInternal(int x, int y, cell_t type) {
  if (!IsOffBuffers(x, y)) {
    // Determine which buffer it is on and set that
    if (y < bot_buff->height) {
      bot_buff->SetCellDirect(x, y, type);
    } else {
      top_buff->SetCellDirect(x, y - bot_buff->height, type);
    }
  }
}


// Decrease to limit
inline bool TypeLayer::DecreaseCell(int x, int y, int amount, int limit) {
  y -= bottom_height;
  cell_t prev_val = GetCellInternal(x, y);
  if (prev_val > limit) {
    int new_val = prev_val - amount;
    if (new_val <= limit) {
      SetCellInternal(x, y, limit);
      return true;
    } else {
      SetCellInternal(x, y, (cell_t)new_val);
      return false;
    }
  }
  return true;
}

// Get the total number of buffers used to reach the height of y
inline int TypeLayer::GetBuffCount(int y) {
  return y/LEVEL_HEIGHT;
}

inline bool BlockYEquals(BlockY a, BlockY b) {
  return a.level == b.level && a.y == b.y;
}

#endif
