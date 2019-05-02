#ifndef EDGE_FINDER_H_
#define EDGE_FINDER_H_

#include <assert.h>
#include "int_vec.h"

struct RectEdge {
  IntVec inside;
  IntVec normal;
};

// Only Works for vectors going down and to the left
inline RectEdge QuadrantEdgeFinder(int x, int y, int dx, int dy) {
  assert(dx <= 0 && dy <= 0);
  assert(!(dx == 0 && dy == 0));
  const int x_diff = (y + 1) * dx;
  const int y_diff = (x + 1) * dy;
  if (y_diff < x_diff) {
    return RectEdge{IntVec{x - x_diff/dy, 0}, IntVec{0, -1}};
  } else if (y_diff > x_diff) {
    return RectEdge{IntVec{0, y - y_diff/dx}, IntVec{-1, 0}};
  } else {
    if (x > y) {
      return RectEdge{IntVec{0, 0}, IntVec{0, -1}};
    } else {
      return RectEdge{IntVec{0, 0}, IntVec{-1, 0}};
    }
  }
}

// Gets the external coordinates of the edge of this block
// in the direction of dx, dy from x, y
inline RectEdge GetRectEdge(int x, int y, int dx, int dy, int rect_width, int rect_height) {
  RectEdge result;
  if (dx > 0) {
    if (dy > 0) {
      result = QuadrantEdgeFinder(rect_width - x - 1, rect_height - y - 1, -dx, -dy);
      result.inside = IntVec{(rect_width - 1) - result.inside.x, (rect_height - 1) - result.inside.y};
      result.normal = IntVec{-result.normal.x, -result.normal.y};
    } else {
      result = QuadrantEdgeFinder(rect_width - x - 1, y, -dx, dy);
      result.inside.x = (rect_width - 1) - result.inside.x;
      result.normal.x = -result.normal.x;
    }
  } else {
    if (dy > 0) {
      result = QuadrantEdgeFinder(x, rect_height - y - 1, dx, -dy);
      result.inside.y = (rect_height - 1) - result.inside.y;
      result.normal.y = -result.normal.y;
    } else {
      result = QuadrantEdgeFinder(x, y, dx, dy);
    }
  }
  return result;
}

#endif // EDGE_FINDER_H_
