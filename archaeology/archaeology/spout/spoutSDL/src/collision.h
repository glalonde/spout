#ifndef COLLISION_H_
#define COLLISION_H_

#include <assert.h>
#include "packed_grid/int_vec.h"

struct Collision { 
  bool exists, reverse_x, out_of_scope;
  int x, y; // The point that was collided with
  int dir_x, dir_y; // The direction "out" of the collision
};

inline Collision MakeCollision(bool reverse_x, int x, int y, int dir_x, int dir_y) {
  return { true, reverse_x, false, x, y, dir_x, dir_y};
}

inline Collision MakeNonCollision() {
  return { false, false, false, -1, -1, 0, 0};
}

// Out of scope either up or down encoded in the y component of dir: dir_y
inline Collision MakeOutOfScopeCollision(int off_y, int x, int y) {
  assert(off_y == -1 || off_y == 1);
  return { false, false, true, x, y, 0, off_y};
}

inline Collision MakeOutOfScopeCollision() {
  return { false, false, true, 0, 0, 0, 0};
}



/*
inline Collision MakeNonCollisionWithData(bool reverse_x, int x, int y, Vec collision_point) {
  return { false, reverse_x, false, x, y, collision_point};
}
*/

#endif
