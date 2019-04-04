#include "src/image_viewer/animated_canvas.h"

AnimatedCanvas::AnimatedCanvas(int window_width, int window_height,
                               int texture_width, int texture_height,
                               double target_fps)
    : target_cycle_time_(FromHz(target_fps)),
      viewer_(window_width, window_height),
      fps_(FromSeconds(1.0), target_fps),
      next_frame_finish_(ClockType::now()) {
  viewer_.SetTextureSize(texture_width, texture_height);
}

Image<PixelType::RGBAU8>* AnimatedCanvas::data() {
  return viewer_.data();
}

ControllerInput AnimatedCanvas::Tick() {
  viewer_.SetDataChanged();
  auto input = viewer_.Update();
  std::this_thread::sleep_until(next_frame_finish_);
  const auto now = ClockType::now();
  fps_.Tick(now - current_frame_finish_);
  current_frame_finish_ = now;
  next_frame_finish_ = current_frame_finish_ + target_cycle_time_;
  return input;
}

double AnimatedCanvas::fps() const {
  return fps_.CurrentEstimate();
}