#include "src/scrolling_manager.h"
#include <numeric>
#include "base/logging.h"

ScrollingManager::ScrollingManager(int buffer_height, int viewport_height)
    : buffer_height_(buffer_height),
      viewport_height_(viewport_height),
      screen_bottom_(0),
      visible_buffers_(viewport_height_ / buffer_height_ +
                       1 /* max number of visible buffers */) {
  UpdateVisibleBuffers();
}

void ScrollingManager::UpdateHeight(int screen_bottom) {
  CHECK_GE(screen_bottom, 0);
  screen_bottom_ = screen_bottom;
  UpdateVisibleBuffers();
}

void ScrollingManager::UpdateVisibleBuffers() {
  int lowest_visible_buffer = screen_bottom_ / buffer_height_;
  int highest_visible_buffer =
      (screen_bottom_ + viewport_height_ - 1) / buffer_height_;
  num_visible_buffers_ = highest_visible_buffer - lowest_visible_buffer + 1;
  for (int i = 0; i < num_visible_buffers_; ++i) {
    visible_buffers_[i] = lowest_visible_buffer + i;
  }
}

int ScrollingManager::GetBufferIndex(int screen_bottom) const {
  return screen_bottom / buffer_height_;
}
