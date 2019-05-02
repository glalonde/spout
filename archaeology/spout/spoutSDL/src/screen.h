#ifndef CELL_GRAPHICS_H_
#define CELL_GRAPHICS_H_
#include "types.h"
#include "constants.h"
#include <cstring>
#include <vector>

extern char font8x8_basic[128][8];

template<class T>
class Screen {
  public:
    static const int width = GRID_WIDTH;
    static const int height = GRID_HEIGHT;
    static const int max_val = (1 << (8*sizeof(T))) - 1;

    int screen_bottom;
    T (*cell_buffer)[GRID_WIDTH];
    static const int buffer_size = width*height*sizeof(T);

    Screen();
    Screen(T Cells[GRID_HEIGHT][GRID_WIDTH]);
    void Reset();
  
    void SetCellRelative(int x, int y, T value);
    void AddToCellRelative(int x, int y, T value);
    void SetCell(int x, int y, T value);
    void SetCell(double x, double y, T value);
    void AddToCell(int x, int y, T value);
    void AddToCell(double x, double y, T value);
    void SetCellDirect(int x, int y, T value);
    void DrawChar(char c, int x, int y, T value);
    void DrawString(const char* str, int x, int y, T value, bool center);
    void DrawLine(int x, int y, int x1, int y1, T value);
    void SyncHeight(int screen_bottom);
    void Clear();
};

template<class T>
inline Screen<T>::Screen() : screen_bottom(0), cell_buffer(NULL) {}

template<class T>
inline Screen<T>::Screen(T Cells[GRID_HEIGHT][GRID_WIDTH]) : screen_bottom(0), cell_buffer(Cells) {}

template<class T>
inline void Screen<T>::Reset() {
  Clear();
  screen_bottom = 0;
}

template<class T>
inline void Screen<T>::SyncHeight(int screen_bottom) {
  this->screen_bottom = screen_bottom;
}

template<class T>
inline void Screen<T>::Clear() {
  memset(cell_buffer, 0, buffer_size);
}

template<class T>
inline void Screen<T>::SetCellRelative(int x, int y, T value) {
  if ((y >= 0) && (y < GRID_HEIGHT) && (x >= 0) && (x < GRID_WIDTH)) {
    cell_buffer[y][x] = value;
  }
}

template<class T>
inline void Screen<T>::AddToCellRelative(int x, int y, T value) {
  if ((y >= 0) && (y < GRID_HEIGHT) && (x >= 0) && (x < GRID_WIDTH)) {
    int new_val = cell_buffer[y][x] + value;
    if (new_val > max_val) {
      cell_buffer[y][x] = max_val;
    } else {
      cell_buffer[y][x] = new_val;
    }
  }
}

template<class T>
inline void Screen<T>::SetCellDirect(int x, int y, T value) {
  if ((x >= 0) && (x < GRID_WIDTH) && (y < GRID_HEIGHT) && (y >= 0)) {
    cell_buffer[y][x] = value;
  }
}

template<class T>
inline void Screen<T>::SetCell(int x, int y, T value) {
  y -= screen_bottom;
  SetCellRelative( x, y, value);
}

template<class T>
inline void Screen<T>::SetCell(double x, double y, T value) {
  SetCellRelative((int)x, (int)y - screen_bottom, value);
}

template<class T>
inline void Screen<T>::AddToCell(int x, int y, T value) {
  y -= screen_bottom;
  AddToCellRelative( x, y, value);
}

template<class T>
inline void Screen<T>::AddToCell(double x, double y, T value) {
  AddToCellRelative((int)x, (int)y - screen_bottom, value);
}


// Bresenham's line algorithm.
template<class T>
inline void Screen<T>::DrawLine(int x, int y, int x1, int y1, T value) {
  int dx = abs(x1 - x);
  int dy = abs(y1 - y);
  int x_step = (x > x1) ? -1 : 1;
  int y_step = (y > y1) ? -1 : 1;
  if (dx > dy) {
    int err = dx;
    while (x != x1) {
      SetCell(x, y, value);
      err -= 2*dy;
      if (err < 0) {
        y += y_step;
        err += 2*dx;
      }
      x += x_step;
    }
  } else {
    int err = dy;
    while (y != y1) {
      SetCell(x, y, value);
      err -= 2*dx;
      if (err < 0) {
        x += x_step;
        err += 2*dy;
      }
      y += y_step;
    }
  }
}

template<class T>
inline void Screen<T>::DrawChar(char c, int x, int y, T value) {
  char* char_start = font8x8_basic[(int)c];
  for (int dx = 0; dx < FONT_HEIGHT; dx++) {
    for (int dy = 0; dy < FONT_WIDTH; dy++) {
      if (char_start[dy] & 0x1 << dx) {
        SetCellRelative(x + dx, y + FONT_HEIGHT - dy, value);
      }
    }
  }
}

template<class T>
inline void Screen<T>::DrawString(const char* str, int x, int y, T value, bool center) {
  if (center) {
    int pix_width = FONT_WIDTH * strlen(str);
    x -= pix_width / 2;
  }
  for (unsigned int i = 0; i < strlen(str); i++) {
    DrawChar(str[i], x + i*FONT_WIDTH, y, value);
  }
}

#endif // CELL_GRAPHICS_H_
