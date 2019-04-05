#pragma once

#include "absl/container/fixed_array.h"

class ScrollingManager {
 public:
  // Number of rows in each buffer and number of rows in the viewport
  ScrollingManager(int buffer_height, int viewport_height);
  void UpdateHeight(int screen_bottom);

  // Only the first `num_visible_buffers()` of this array are valid indices.
  const absl::FixedArray<int>& visible_buffers() const {
    return visible_buffers_;
  }

  int num_visible_buffers() const {
    return num_visible_buffers_;
  }

  int max_visible_buffers() const {
    return visible_buffers_.size();
  }

 private:
  int GetBufferIndex(int screen_bottom) const;
  void UpdateVisibleBuffers();

  // Parameters
  int buffer_height_;
  int viewport_height_;

  // State
  int screen_bottom_;
  int num_visible_buffers_;
  absl::FixedArray<int> visible_buffers_;
};
