#include "graphics/animated_canvas.h"

AnimatedCanvas::AnimatedCanvas(int window_width, int window_height,
                               int texture_width, int texture_height,
                               double target_fps)
    : target_cycle_time_(FromHz(target_fps)),
      viewer_(window_width, window_height),
      fps_(FromSeconds(1.0), target_fps) {
  viewer_.SetTextureSize(texture_width, texture_height);
}

Image<PixelType::RGBAU8>* AnimatedCanvas::data() {
  return viewer_.data();
}

ControllerInput AnimatedCanvas::Tick(Duration* dt) {
  viewer_.SetDataChanged();
  auto input = viewer_.Update();
  auto delta = fps_.Tick();
  if (dt) {
    *dt = delta;
  }
  return input;
}

double AnimatedCanvas::fps() const {
  return fps_.CurrentEstimate();
}
