#ifndef PACKED_GRID_H_
#define PACKED_GRID_H_

#include <stdint.h>   
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <iostream>

#include "../constants.h"
#include "../collision.h"
#include "int_vec.h"

typedef uint64_t block_t;

class PackedGrid {
public:
  // The cells are 64 bits.
  static const int width = LEVEL_WIDTH;
  static const int height = LEVEL_HEIGHT;

  // Blocks are 8 bits high
  static const int BLOCK_HEIGHT = 8;

  // Blocks are 4 bits wide
  static const int BLOCK_WIDTH = 8;

  // The size, in blocks, of the grid
  static const int BLOCKS_WIDE = (width + BLOCK_WIDTH - 1) / BLOCK_WIDTH;
  static const int BLOCKS_HIGH = (height + BLOCK_HEIGHT - 1) / BLOCK_HEIGHT;

  // The main bit board.
  block_t data[BLOCKS_WIDE][BLOCKS_HIGH];

  /* FUNCTIONS */
  PackedGrid() {
    //assert(LEVEL_WIDTH%BLOCK_WIDTH == 0);
    //assert(LEVEL_HEIGHT%BLOCK_HEIGHT == 0);
  }

  static Collision IntraBlockBresenham(block_t* block, IntVec curr, IntVec end);

  Collision CheckPair(IntVec curr, IntVec end); 

  // Set a cell with coordinates relative to the whole grid
  inline void SetCell(int x, int y) {
    assert(x >= 0 && x < width);
    assert(y >= 0 && y < height);
    data[x / BLOCK_WIDTH][y / BLOCK_HEIGHT] |= GetMask(x%BLOCK_WIDTH, y%BLOCK_HEIGHT);
  }

  // Set a cell with coordinates relative to the whole grid
  inline void ClearCell(int x, int y) {
    assert(x >= 0 && x < width);
    assert(y >= 0 && y < height);
    data[x / BLOCK_WIDTH ][y / BLOCK_HEIGHT] &= ~GetMask(x%BLOCK_WIDTH, y%BLOCK_HEIGHT);
  }

  // Get the value of a cell with coordinates relative to the whole grid
  inline bool GetCell(int x, int y) {
    assert(x >= 0 && x < width);
    assert(y >= 0 && y < height);
    return data[x / BLOCK_WIDTH][y / BLOCK_HEIGHT] & GetMask(x%BLOCK_WIDTH, y%BLOCK_HEIGHT);
  }

  inline block_t GetBlock(int x, int y) {
    assert(x >= 0 && x < width);
    assert(y >= 0 && y < height);
    return data[x][y];
  }

  inline void Reset() {
    ClearAll();
  }

  inline void ClearAll() {
    memset(data, 0, sizeof(block_t)*BLOCKS_WIDE*BLOCKS_HIGH);
  }

  inline void SetAll() {
    memset(data, 0xFF, sizeof(block_t)*BLOCKS_WIDE*BLOCKS_HIGH);
  }

  /*
  inline bool IsEmpty() {
    // If there are an even number of blocks high.
    if (BLOCKS_HIGH % 2 == 0) {
      for (int x = 0; x < BLOCKS_WIDE; x++) {
        for (int y = 0; y < BLOCKS_HIGH; y += 2) {
          if (*((uint64_t*)&data[x][y]) != 0) {
            return false;
          }
        }
      }
    } else {
      for (int x = 0; x < BLOCKS_WIDE; x++) {
        for (int y = 0; y < BLOCKS_HIGH; y += 2) {
          if (*((uint64_t*)&data[x][y]) != 0) {
            return false;
          }
        }
        // If it wasn't even, we need to check the last block.
        if (data[x][BLOCKS_HIGH - 1] != 0) {
          return false;
        }
      }
    }
    return true;
  }
  */
  inline void FillRect(int x, int y, int rwidth, int rheight) {
    for (int j = y; j < y + rheight; j++) {
      for (int i = x; i < x + rwidth; i++) {
        SetCell(i, j);
      }
    }
  }

  inline void ClearRect(int x, int y, int rwidth, int rheight) {
    for (int j = y; j < y + rheight; j++) {
      for (int i = x; i < x + rwidth; i++) {
        ClearCell(i, j);
      }
    }
  }

  inline static IntVec GlobalToBlock(IntVec abs_cell_pos) {
    return IntVec{abs_cell_pos.x/BLOCK_WIDTH, abs_cell_pos.y/BLOCK_HEIGHT};
  }

  inline static IntVec GlobalToCell(IntVec abs_cell_pos) {
    return IntVec{abs_cell_pos.x%BLOCK_WIDTH, abs_cell_pos.y%BLOCK_HEIGHT};
  }

  static inline block_t CheckBlockCell(block_t block, int x, int y) {
    assert(x >= 0 && x < BLOCK_WIDTH);
    assert(y >= 0 && y < BLOCK_HEIGHT);
    return block & GetMask(x, y);
  }

  // Get the mask for a coordinate relative to the block
  static inline block_t GetMask(int x, int y) {
    return (block_t)0x1 << ((x) * BLOCK_HEIGHT + y);
  }
};
#endif // PACKED_GRID_H_
