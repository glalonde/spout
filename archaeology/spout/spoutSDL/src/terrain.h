#ifndef TERRAIN_H_
#define TERRAIN_H_

#include <vector>
#include <iostream>
#include "constants.h"
#include "types.h"
#include "collision.h"
#include "mobile_object.h"
#include "vec.h"
#include "type_layer.h"
#include "palette.h"

#include "packed_grid/int_vec.h"
//#include "packed_grid/edge_finder.h"
#include "packed_grid/packed_grid_64.h"

class Terrain : public TypeLayer {
  public:
    int bottom_level;
    Terrain ();
    virtual void Reset();
  
    void PrintOcc();
    void MakeLevel(TileBuffer* level_buffer, int level_num);
    void MakeWebbingLevel(TileBuffer* level_buffer, int level_num);
    void MakeSpeedTestLevel(TileBuffer* level_buffer, int level_num);
    void GenerateWebbingLevel(TileBuffer* level_buffer, int n_lines);
    void GenerateRectLevel(TileBuffer* level_buffer, int max_dimension, int num_vacancies);

    //Collision CheckBuffer(TileBuffer* buff, int x1, int y1, int x2, int y2);
    Collision GetCollisionFast(MobileObject* object);
    bool IsFull(int x, int y);
    void Remove(int x, int y);
    void Damage(int x, int y, vec_dimension speed);
    void MarkEdges(TileBuffer* buff);
  
    // Extended
    virtual void SwapBuffers();
    virtual void UnSwapBuffers();
};

inline bool Terrain::IsFull(int x, int y) {
  // If it is off in the x direction, like hitting the wall, it "collides"
  if (IsOffBuffersX(x)) {
    return true;
  }
  return TypeLayer::GetOccupancyDirect(x, y);
}

inline void Terrain::Remove(int x, int y) {
  TypeLayer::SetCell(x, y, TERRAIN_TYPES::EMPTY);
}

inline void Terrain::Damage(int x, int y, vec_dimension speed) {
  if (DecreaseCell(x, y, (uint8_t)(speed*SPEED_DAMAGE), TERRAIN_TYPES::EMPTY)) {
    Remove(x, y);
  }
}

/*
// Returns true if it ran off the top or bottom
inline Collision Terrain::CheckBuffer(TileBuffer* buff, int x1, int y1, int x2, int y2) {
  assert(x1 >= 0 && x1 < buff->width);
  assert(y1 >= 0 && y1 < buff->height);

  if (x2 < 0 || x2 >= buff->width || y2 < 0 || y2 >= buff->height) {
    // Find edge. Check pair(x1, y1, edgex, edgey)
    RectEdge edge = GetRectEdge(x1, y1, x2, y2, LEVEL_WIDTH, LEVEL_HEIGHT);
    Collision coll = buff->CheckPair(IntVec{x1, y1}, edge.inside);

    if (coll.exists) {
      return coll;
    } else {

      // If it didn't find a real collision, then it must have made it all the way to the edge
      if (edge.normal.x != 0) {
        assert(edge.normal.y == 0);
        return MakeCollision(true, edge.inside.x + edge.normal.x, edge.inside.y, -edge.normal.x, 0);
      } else {
        // If it didn't hit the left or right, it must have hit top or bottom
        assert(edge.normal.y == 1 || edge.normal.y == -1);
        return MakeOutOfScopeCollision(edge.normal.y, edge.inside.x, edge.inside.y);
      } 
    }
  } else {
    return buff->CheckPair(IntVec{x1, y1}, IntVec{x2, y2});
  }
}
*/


inline void Terrain::MarkEdges(TileBuffer* buff) {

  for (int y = 0; y < buff->height - 0; y++) {
    for (int x = 0; x < buff->width - 0; x++) {
      if (buff->GetCell(x, y) != TERRAIN_TYPES::EMPTY) {
        if ((buff->GetCell(x + 1, y) == TERRAIN_TYPES::EMPTY) ||
            (buff->GetCell(x - 1, y) == TERRAIN_TYPES::EMPTY) ||
            (buff->GetCell(x, y + 1) == TERRAIN_TYPES::EMPTY) ||
            (buff->GetCell(x, y - 1) == TERRAIN_TYPES::EMPTY)) {
          buff->SetCellDirect(x, y, TERRAIN_TYPES::EDGE);
          continue;
        }
      }
    }
  }
}

inline Collision Terrain::GetCollisionFast(MobileObject* object) {
  int x1 = (int)object->prev_pos.x;
  int y1 = (int)object->prev_pos.y - this->bottom_height;
  int x2 = (int)object->pos.x;
  int y2 = (int)object->pos.y - this->bottom_height;

  assert(!IsOffBuffersX(x1));
  assert(!IsOffBuffersY(y1));

  if (x1 == x2 && y1 == y2) {
    return MakeNonCollision();
  } else if (IsOffBuffersY(y1)) {
    return MakeOutOfScopeCollision();
  }

  int block_x = (x1 >= 0) ? x1 / PackedGrid::BLOCK_WIDTH : -1;
  BlockY block_y = (y1 >= 0) ? GetBlockY(y1) : BlockY{.level = -1, .y = 0};
  int end_block_x = (x2 >= 0) ? x2 / PackedGrid::BLOCK_WIDTH : -1;
  BlockY end_block_y = (y2 >= 0) ? GetBlockY(y2) : BlockY{.level = -1, .y = 0};
  block_t curr_block;
  curr_block = TypeLayer::GetBlock(block_x, block_y);

  bool check_block = curr_block != 0;

  if (!check_block && block_x == end_block_x && BlockYEquals(block_y, end_block_y)) {
    return MakeNonCollision();
  }

  int dx = abs(x2 - x1);
  int dy = abs(y2 - y1);
  int x_step = (x1 > x2) ? -1 : 1;
  int y_step = (y1 > y2) ? -1 : 1;
  BlockY temp_y;
  int temp_x;
  int double_dx = 2*dx;
  int double_dy = 2*dy;
  int err;
  if (dx > dy) {
    err = dx;
    while (x1 != x2 || y1 != y2) {
      if (err < dy) {
        y1 += y_step;
        if (IsOffBuffersY(y1)) {
          return MakeOutOfScopeCollision();
        }

        temp_y = GetBlockY(y1);
        if (!BlockYEquals(temp_y, block_y)) {
          block_y = temp_y;
          curr_block = TypeLayer::GetBlock(block_x, block_y);
          check_block = curr_block != 0;
          if (!check_block && block_x == end_block_x && BlockYEquals(block_y, end_block_y)) {
            return MakeNonCollision();
          }
        }

        if (y1 < bot_buff->height) {
          if (check_block && PackedGrid::CheckBlockCell(curr_block, x1%PackedGrid::BLOCK_WIDTH, y1%PackedGrid::BLOCK_HEIGHT)) {
            return MakeCollision(false, x1, y1 + bottom_height, 0, -y_step);
          }
        } else {
          if (check_block && PackedGrid::CheckBlockCell(curr_block, x1%PackedGrid::BLOCK_WIDTH, (y1 - bot_buff->height)%PackedGrid::BLOCK_HEIGHT)) {
            return MakeCollision(false, x1, y1 + bottom_height, 0, -y_step);
          }
        }

        err += double_dx;
      } else {
        x1 += x_step;
        if (IsOffBuffersX(x1)) {
          return MakeCollision(true, x1, y1 + bottom_height, -x_step, 0);
        }

        temp_x = x1 / PackedGrid::BLOCK_WIDTH;
        if (temp_x != block_x) {
          block_x = temp_x;
          curr_block = TypeLayer::GetBlock(block_x, block_y);
          check_block = curr_block != 0;
          if (!check_block && block_x == end_block_x && BlockYEquals(block_y, end_block_y)) {
            return MakeNonCollision();
          }
        }
        if (y1 < bot_buff->height) {
          if (check_block && PackedGrid::CheckBlockCell(curr_block, x1%PackedGrid::BLOCK_WIDTH, y1%PackedGrid::BLOCK_HEIGHT)) {
            return MakeCollision(true, x1, y1 + bottom_height, -x_step, 0);
          }
        } else {
          if (check_block && PackedGrid::CheckBlockCell(curr_block, x1%PackedGrid::BLOCK_WIDTH, (y1 - bot_buff->height)%PackedGrid::BLOCK_HEIGHT)) {
            return MakeCollision(true, x1, y1 + bottom_height, -x_step, 0);
          }
        }
        err -= double_dy;
      }
    }
  } else {
    err = dy;
    while (x1 != x2 || y1 != y2) {
      if (err < dx) {
        x1 += x_step;
        if (IsOffBuffersX(x1)) {
          return MakeCollision(true, x1, y1 + bottom_height, -x_step, 0);
        }

        temp_x = x1 / PackedGrid::BLOCK_WIDTH;
        if (temp_x != block_x) {
          block_x = temp_x;
          curr_block = TypeLayer::GetBlock(block_x, block_y);
          check_block = curr_block != 0;
          if (!check_block && block_x == end_block_x && BlockYEquals(block_y, end_block_y)) {
            return MakeNonCollision();
          }
        }
        if (y1 < bot_buff->height) {
          if (check_block && PackedGrid::CheckBlockCell(curr_block, x1%PackedGrid::BLOCK_WIDTH, y1%PackedGrid::BLOCK_HEIGHT)) {
            return MakeCollision(true, x1, y1 + bottom_height, -x_step, 0);
          }
        } else {
          if (check_block && PackedGrid::CheckBlockCell(curr_block, x1%PackedGrid::BLOCK_WIDTH, (y1 - bot_buff->height)%PackedGrid::BLOCK_HEIGHT)) {
            return MakeCollision(true, x1, y1 + bottom_height, -x_step, 0);
          }
        }
        err += double_dy;
      } else {
        y1 += y_step;

        if (IsOffBuffersY(y1)) {
          return MakeOutOfScopeCollision();
        }

        temp_y = GetBlockY(y1);

        if (!BlockYEquals(temp_y, block_y)) {
          block_y = temp_y;
          curr_block = TypeLayer::GetBlock(block_x, block_y);
          check_block = curr_block != 0;
          if (!check_block && block_x == end_block_x && BlockYEquals(block_y, end_block_y)) {
            return MakeNonCollision();
          }
        }

        if (y1 < bot_buff->height) {
          if (check_block && PackedGrid::CheckBlockCell(curr_block, x1%PackedGrid::BLOCK_WIDTH, y1%PackedGrid::BLOCK_HEIGHT)) {
            return MakeCollision(false, x1, y1 + bottom_height, 0, -y_step);
          }
        } else {
          if (check_block && PackedGrid::CheckBlockCell(curr_block, x1%PackedGrid::BLOCK_WIDTH, (y1 - bot_buff->height)%PackedGrid::BLOCK_HEIGHT)) {
            return MakeCollision(false, x1, y1 + bottom_height, 0, -y_step);
          }
        }

        err -= double_dx;
      }
    }
  }

  return MakeNonCollision();
}

#endif
