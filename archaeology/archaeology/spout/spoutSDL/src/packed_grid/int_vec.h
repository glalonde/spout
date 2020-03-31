#ifndef INT_VEC_H_
#define INT_VEC_H_

struct IntVec {
  int x;  // The x-coordinate of the vector.
  int y;  // The y-coordinate of the vector.
};

struct BoolIntVec {
  bool test;
  IntVec vec;
};

inline IntVec VecAdd(IntVec lh, IntVec rh) {
  return IntVec{lh.x + rh.x, lh.y + rh.y};
}
inline IntVec VecSubtract(IntVec lh, IntVec rh) {
  return IntVec{lh.x - rh.x, lh.y - rh.y};
}
inline bool VecEquals(IntVec lh, IntVec rh) {
  return (lh.x == rh.x) && (lh.y == rh.y);
}

#endif // INT_VEC_H
