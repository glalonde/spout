#include "src/scrolling_manager.h"
#include <numeric>
#include "base/logging.h"

ScrollingManager::ScrollingManager(int buffer_height, int viewport_height)
    : buffer_height_(buffer_height),
      viewport_height_(viewport_height),
      screen_bottom_(0) {
  UpdateVisibleBuffers();
}

void ScrollingManager::UpdateHeight(int screen_bottom) {
  CHECK_GE(screen_bottom, 0);
  screen_bottom_ = screen_bottom;
  UpdateVisibleBuffers();
}

void ScrollingManager::UpdateVisibleBuffers() {
  lowest_visible_buffer_ = screen_bottom_ / buffer_height_;
  highest_visible_buffer_ =
      (screen_bottom_ + viewport_height_ - 1) / buffer_height_;
}

int ScrollingManager::GetBufferIndex(int screen_bottom) const {
  return screen_bottom / buffer_height_;
}

ScrollingCanvas::ScrollingCanvas(
    Vector2i level_dimensions /* width, height */, int viewport_height,
    std::function<void(int i, Image<PixelType::RGBAU8>*)> level_gen_function)
    : level_dimensions_(level_dimensions),
      manager_(level_dimensions.y(), viewport_height),
      level_gen_(std::move(level_gen_function)) {}

void ScrollingCanvas::SetHeight(int screen_bottom) {
  manager_.UpdateHeight(screen_bottom);
}

void ScrollingCanvas::Render(Image<PixelType::RGBAU8>* viewport) {
  while (manager_.highest_visible_buffer() + 1 >= buffers_.size()) {
    MakeLevelBuffer(buffers_.size());
  }
  int viewport_bottom = 0;
  int start_row;
  int num_rows;
  for (int i = manager_.lowest_visible_buffer();
       i <= manager_.highest_visible_buffer(); ++i) {
    // Copy data
    manager_.VisibleRows(i, &start_row, &num_rows);
    viewport->block(viewport_bottom, 0, num_rows, viewport->cols()) =
        buffers_[i].block(start_row, 0, num_rows, viewport->cols());
    viewport_bottom += num_rows;
  }
}

void ScrollingCanvas::MakeLevelBuffer(int i) {
  CHECK_EQ(buffers_.size(), i);
  buffers_.emplace_back(level_dimensions_.y(), level_dimensions_.x());
  level_gen_(i, &buffers_.back());
}
