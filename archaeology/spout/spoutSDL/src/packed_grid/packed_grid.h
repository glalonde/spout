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
#include "edge_finder.h"

typedef uint32_t block_t;

class PackedGrid {
public:
  // The cells are 32 bits.
  static const int width = LEVEL_WIDTH;
  static const int height = LEVEL_HEIGHT;

  // Blocks are 8 bits high
  static const int BLOCK_HEIGHT = 8;

  // Blocks are 4 bits wide
  static const int BLOCK_WIDTH = 4;

  // The size, in blocks, of the grid
  static const int BLOCKS_WIDE = (width + BLOCK_WIDTH - 1) / BLOCK_WIDTH;
  static const int BLOCKS_HIGH = (height + BLOCK_HEIGHT - 1) / BLOCK_HEIGHT;

  // The main bit board.
  block_t data[BLOCKS_WIDE][BLOCKS_HIGH];

  /* FUNCTIONS */
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
    memset(data, ~((block_t)0x0), sizeof(block_t)*BLOCKS_WIDE*BLOCKS_HIGH);
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

  void Print() {
    for (int y = height - 1; y > -1; y--) {
      for (int x = 0; x < width; x++) {
        printf("%d", GetCell(x, y) ? 1 : 0);
      }
      printf("\n");
    } 
  }

  inline static void PrintBlock(block_t block) {
    for (int y = BLOCK_HEIGHT - 1; y > -1; y--) {
      for (int x = 0; x < BLOCK_WIDTH; x++) {
        printf("%d", (block & GetMask(x, y)) ? 1 : 0);
      }
      printf("\n");
    } 
  }

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

  static inline block_t CheckBlockCell(block_t* block, int x, int y) {
    assert(x >= 0 && x < BLOCK_WIDTH);
    assert(y >= 0 && y < BLOCK_HEIGHT);
    return *block & GetMask(x, y);
  }

  // Get the mask for a coordinate relative to the block
  static inline block_t GetMask(int x, int y) {
    return (block_t)0x1 << ((x) * BLOCK_HEIGHT + y);
  }

  // Get the index of the block in the main block array
  static inline int GetBlockIndex(int x, int y) {
    int index = (x / BLOCK_WIDTH) * BLOCKS_HIGH + (y / BLOCK_HEIGHT);
    assert(index >= 0);
    assert(index < BLOCKS_WIDE*BLOCKS_HIGH);
    return index;
  }
};

inline Collision PackedGrid32::IntraBlockBresenham(block_t* block, IntVec curr, IntVec end) {
  IntVec dir = VecSubtract(end, curr);

  IntVec step;
  if (dir.x >= 0) {
    step.x = 1;
  } else {
    step.x = -1;
    dir.x = -dir.x; // needs to be positive.
  }
  if (dir.y >= 0) {
    step.y = 1;
  } else {
    step.y = -1;
    dir.y = -dir.y;
  }

  // All points must be strictly inside the bounds of a block
  assert(curr.x >= 0 && curr.x < BLOCK_WIDTH);
  assert(curr.y >= 0 && curr.y < BLOCK_HEIGHT);
  assert(end.x >= 0 && end.x < BLOCK_WIDTH);
  assert(end.y >= 0 && end.y < BLOCK_HEIGHT);

  if (dir.x > dir.y) {
    int err = dir.x;
    while (!VecEquals(curr, end)) {
      if (err < dir.y) {
        curr.y += step.y;
        if (*block & GetMask(curr.x, curr.y)) {
          return MakeCollision(false, curr.x, curr.y, 0, -step.y);
        }
        err += 2*dir.x;
      } else {
        curr.x += step.x;
        if (*block & GetMask(curr.x, curr.y)) {
          return MakeCollision(true, curr.x, curr.y, -step.x, 0);
        }
        err -= 2*dir.y;
      }
    }
  } else {
    int err = dir.y;
    while (!VecEquals(curr, end)) {
      if (err <= dir.x) {
        curr.x += step.x;
        if (*block & GetMask(curr.x, curr.y)) {
          return MakeCollision(true, curr.x, curr.y, -step.x, 0);
        }
        err += 2*dir.y;
      } else {
        curr.y += step.y;
        if (*block & GetMask(curr.x, curr.y)) {
          return MakeCollision(false, curr.x, curr.y, 0, -step.y);
        }
        err -= 2*dir.x;
      }
    }
  }
  return MakeNonCollision();
}

// On takes pairs that are on the inside of the border.
inline Collision PackedGrid32::CheckPair(IntVec curr, IntVec end) {
  assert(curr.x >= 0 && curr.x < this->width);
  assert(curr.y >= 0 && curr.y < this->height);
  assert(end.x >=  0 && end.x < this->width);
  assert(end.y >=  0 && end.y < this->height);

  IntVec curr_block = GlobalToBlock(curr); // The current block
  IntVec end_block = GlobalToBlock(end);  // The final bloc;
  IntVec cell = GlobalToCell(curr); // The current point relative to the current_block

  RectEdge edge;
  Collision coll;
  bool is_done = false;

  while (true) {
    if (VecEquals(curr_block, end_block)) {
      // This is the final block, so stash the target in edge.inside
      // We know it won't actually hit the edge, and it will terminate before that.
      is_done = true;
      edge.inside = GlobalToCell(end);
      edge.normal = IntVec{0, 0};
    } else {
      edge = GetRectEdge(cell.x, cell.y, end.x - curr.x, end.y - curr.y, BLOCK_WIDTH, BLOCK_HEIGHT);
    }

    // Get the block-level collision.
    //coll = DoBlock(&data[curr_block.x][curr_block.y], cell, edge.inside);
    if (data[curr_block.x][curr_block.y] == 0) {
      coll = MakeNonCollision();
    } else {
      coll = IntraBlockBresenham(&data[curr_block.x][curr_block.y], cell, edge.inside);
    }

    // If that block found a collision then we transform to global points and return
    if (coll.exists) {
      IntVec corner = VecSubtract(curr, cell);
      coll.x += corner.x;
      coll.y += corner.y;
      coll.dir_x = edge.normal.x;
      coll.dir_y = edge.normal.y;
      return coll;

    } else if (is_done) {
      return MakeNonCollision();
    } else {
      curr = VecSubtract(curr, cell); // Translate to block corner
      curr = VecAdd(curr, edge.inside); // Translate to edge
      curr = VecAdd(curr, edge.normal); // Translate to adjacent

      curr_block = GlobalToBlock(curr); //VecAdd(curr_block, edge.normal);
      cell = GlobalToCell(curr);

      // This *should* be checking somewhere on the inside border of the next block.
      // i.e. the first cell in the next DoBlock sequence.
      if (data[curr_block.x][curr_block.y] & GetMask(cell.x, cell.y)) {
        return MakeCollision(true, curr.x, curr.y, -edge.normal.x, -edge.normal.y);
      }
    }
  }
}


#endif // PACKED_GRID_H_
