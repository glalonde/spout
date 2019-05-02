#include "tile_buffer.h"

TileBuffer::TileBuffer() : id(-1), buffer_size(width*height*sizeof(cell_t)) {
  InitColorMap(COLORS::RED, COLORS::BLACK, color_map, NUM_TYPES - 1);
  color_map[TERRAIN_TYPES::FULL] = COLORS::DARKER_GREY;
}

TileBuffer::TileBuffer(pixel_t start_color, pixel_t end_color) : id(-1), buffer_size(width*height*sizeof(cell_t)) {
  InitColorMap(start_color, end_color, color_map, NUM_TYPES - 1);
  color_map[TERRAIN_TYPES::FULL] = COLORS::DARKER_GREY;
}

void TileBuffer::Reset() {
  this->Clear(); //TODO might not be necessary, whatevs
}

// Drawing methods
void TileBuffer::Clear() {
  SetAll(0);
  occupancy.Reset();
}

void TileBuffer::SetAll(cell_t type) {
  memset(&cells, type, buffer_size);
  if (type != cell_types::EMPTY) {
    occupancy.SetAll();
  } else {
    occupancy.ClearAll();
  }
}

// Bresenham's line algorithm.
void TileBuffer::DrawLine(int x, int y, int x1, int y1, cell_t type) {
  const int dx = abs(x1 - x);
  const int dy = abs(y1 - y);
  const int x_step = (x > x1) ? -1 : 1;
  const int y_step = (y > y1) ? -1 : 1;
  if (dx > dy) {
    int err = dx;
    while (x != x1) {
      SetCellDirect(x, y, type);
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
      SetCellDirect(x, y, type);
      err -= 2*dx;
      if (err < 0) {
        x += x_step;
        err += 2*dy;
      }
      y += y_step;
    }
  }
}

void TileBuffer::FillRect(int x, int y, int width, int height, cell_t type) {
  for (int j = y; j < y + height; j++) {
    for (int i = x; i < x + width; i++) {
      SetCellDirect(i, j, type);
    }
  }
}

void TileBuffer::CoolRect(int left, int bot, int box_width, int box_height, int n_lines, cell_t type) {
  int x_inc = (box_width >= n_lines) ? (box_width / n_lines) : 1;
  int y_inc = (box_height >= n_lines) ? box_height / n_lines : 1;
  int right = left + box_width - 1;
  int top = bot + box_height - 1;
  DrawLine(left, bot, right, bot, type);
  DrawLine(left, bot, left, top, type);
  DrawLine(right, bot, right, top, type);
  DrawLine(left, top, right, top, type);
  for (int i = 1; i < n_lines; i ++) {
    DrawLine(left + i*x_inc, bot, right, bot + i*y_inc, type);
    DrawLine(left, bot + i*y_inc, left + i*x_inc, top, type);
  }
}


void TileBuffer::Draw(Screen<counter_t>* screen, int origin_y) {
  const int bottom = (screen->screen_bottom > origin_y) ? screen->screen_bottom : origin_y;
  const int top = ((screen->screen_bottom + screen->height) > (origin_y + this->height))? (origin_y + this->height) : (screen->screen_bottom + screen->height);

  int corner_x;
  int corner_y;

  int start_line = bottom - origin_y;
  int first_block = start_line / PackedGrid::BLOCK_HEIGHT ;

  // If there is a bit at the very start in between blocks
  if (start_line % PackedGrid::BLOCK_HEIGHT > 0) {
    first_block ++;
    for (int y = start_line; y < first_block * PackedGrid::BLOCK_HEIGHT; y++) {
      for (int x = 0; x < GRID_WIDTH; x++) {
        cell_t cell = GetCellDirect(x, y);
        assert(cell >= 0);
        assert(cell < 256);
        screen->SetCell(x, y + origin_y, cell);
      }
    }
  }

  int end_line = top - origin_y;
  int last_block = end_line / PackedGrid::BLOCK_HEIGHT;

  // Do the remainder at the end
  if (end_line % PackedGrid::BLOCK_HEIGHT > 0) {
    for (int y = last_block * PackedGrid::BLOCK_HEIGHT; y < end_line; y++) {
      for (int x = 0; x < GRID_WIDTH; x++) {
        cell_t cell = GetCellDirect(x, y);
        assert(cell >= 0);
        assert(cell < 256);
        screen->SetCell(x, y + origin_y, cell);
      }
    }
  }


  // Do the bulk in the middle
  for (int oy = first_block; oy < last_block; oy++) {
    for (int ox = 0; ox < PackedGrid::BLOCKS_WIDE; ox++) {
      if (occupancy.GetBlock(ox, oy) != 0) {
        corner_x = ox*PackedGrid::BLOCK_WIDTH;
        corner_y = oy*PackedGrid::BLOCK_HEIGHT;
        for (int y = corner_y; y < corner_y + PackedGrid::BLOCK_HEIGHT; y++) {
          for (int x = corner_x; x < corner_x + PackedGrid::BLOCK_WIDTH; x++) {
            cell_t cell = GetCellDirect(x, y);
            assert(cell >= 0);
            assert(cell < 256);
            screen->SetCell(x, y + origin_y, cell);
          }
        }
      }
    }
  }
}
