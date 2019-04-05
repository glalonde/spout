#pragma once

#include "base/logging.h"

class ScrollingManager {
 public:
  // Number of rows in each buffer and number of rows in the viewport
  ScrollingManager(int buffer_height, int viewport_height);
  void UpdateHeight(int screen_bottom);

  int lowest_visible_buffer() const {
    return lowest_visible_buffer_;
  }

  int highest_visible_buffer() const {
    return highest_visible_buffer_;
  }

  int ToBufferFrame(int global_row, int buffer_index) const {
    return global_row - RowOffset(buffer_index);
  }

  int FromBufferFrame(int buffer_row, int buffer_index) const {
    return buffer_row + RowOffset(buffer_index);
  }

  int RowOffset(int buffer_index) const {
    return buffer_index * buffer_height_;
  }

  // Returns buffer-relative start-row, and the number of visible rows.
  void VisibleRows(int buffer_index, int* start_row, int* num_rows) const {
    // Global row offset that is the first row of this buffer
    int offset = RowOffset(buffer_index);

    // Bottom of the viewport, relative to this buffer.
    int screen_bottom_relative = screen_bottom_ - offset;
    // Top of the viewport, relative to this buffer.
    int screen_top_relative = screen_bottom_relative + viewport_height_;

    *start_row = std::clamp(screen_bottom_relative, 0, buffer_height_);
    int end_row = std::clamp(screen_top_relative, 0, buffer_height_);
    *num_rows = end_row - *start_row;
  }

 private:
  int GetBufferIndex(int screen_bottom) const;
  void UpdateVisibleBuffers();

  // Parameters
  int buffer_height_;
  int viewport_height_;

  // State
  int screen_bottom_;
  int lowest_visible_buffer_;
  int highest_visible_buffer_;
};