#pragma once

#include "absl/container/fixed_array.h"

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
