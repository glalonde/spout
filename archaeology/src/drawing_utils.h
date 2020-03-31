

// Recall: X corresponds to column, Y corresponds to row in the image space.
template <class T>
void DrawLine(int x, int y, int x1, int y1, const T& color, Image<T>* data) {
  int dx = std::abs(x1 - x);
  int dy = std::abs(y1 - y);
  int x_step = (x > x1) ? -1 : 1;
  int y_step = (y > y1) ? -1 : 1;
  if (dx > dy) {
    int err = dx;
    while (x != x1) {
      if (!IsInImage(y, x, *data)) {
        return;
      }
      (*data)(y, x) = color;
      err -= 2 * dy;
      if (err < 0) {
        y += y_step;
        err += 2 * dx;
      }
      x += x_step;
    }
  } else {
    int err = dy;
    while (y != y1) {
      if (!IsInImage(y, x, *data)) {
        return;
      }
      (*data)(y, x) = color;
      err -= 2 * dx;
      if (err < 0) {
        x += x_step;
        err += 2 * dy;
      }
      y += y_step;
    }
  }
}

// Same as above but doesn't check if the point is still valid, so potentially
// segfaults.
template <class T>
void UnsafeDrawLine(int x, int y, int x1, int y1, const T& color,
                    Image<T>* data) {
  int dx = std::abs(x1 - x);
  int dy = std::abs(y1 - y);
  int x_step = (x > x1) ? -1 : 1;
  int y_step = (y > y1) ? -1 : 1;
  if (dx > dy) {
    int err = dx;
    while (x != x1) {
      (*data)(y, x) = color;
      err -= 2 * dy;
      if (err < 0) {
        y += y_step;
        err += 2 * dx;
      }
      x += x_step;
    }
  } else {
    int err = dy;
    while (y != y1) {
      (*data)(y, x) = color;
      err -= 2 * dx;
      if (err < 0) {
        x += x_step;
        err += 2 * dy;
      }
      y += y_step;
    }
  }
}
