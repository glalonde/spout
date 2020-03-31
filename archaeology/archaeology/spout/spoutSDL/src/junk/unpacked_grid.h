#ifndef UNPACKED_GRID_H_
#define UNPACKED_GRID_H_

#include <stdint.h>   
#include <assert.h>

class UnpackedGrid {
public:
  // The cells are 32 bits.
  typedef uint8_t cell_t;
  static const int width = LEVEL_WIDTH;
  static const int height = LEVEL_HEIGHT;

  cell_t data[width][height];

  // Set a cell with coordinates relative to the whole grid
  inline void SetCell(int x, int y) {
    assert(x >= 0 && x < width);
    assert(y >= 0 && y < height);
    data[x][y] = true;
  }

  // Set a cell with coordinates relative to the whole grid
  inline void ClearCell(int x, int y) {
    assert(x >= 0 && x < width);
    assert(y >= 0 && y < height);
    data[x][y] = false;
  }

  // Get the value of a cell with coordinates relative to the whole grid
  inline bool GetCell(int x, int y) {
    assert(x >= 0 && x < width);
    assert(y >= 0 && y < height);
    return data[x][y] > 0;
  }

  inline bool IsEmpty() {
    for (int x = 0; x < width; x++) {
      for (int y = 0; y < height; y++) {
        if (data[x][y] != 0) {
          return false;
        }
      }
    }
    return true;
  }
};

#endif // UNPACKED_GRID_H_
