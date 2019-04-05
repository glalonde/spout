#pragma once
#include <thread>
#include "base/time.h"
#include "src/fps_estimator.h"
#include "src/image.h"
#include "src/image_viewer/image_viewer.h"

// Mutate `data` then call `Tick`
class AnimatedCanvas {
 public:
  AnimatedCanvas(int window_width, int window_height, int texture_width,
                 int texture_height, double target_fps);

  Image<PixelType::RGBAU8>* data();

  // Optionally returns the amount of time spent since the previous tick.
  ControllerInput Tick(Duration* dt = nullptr);

  double fps() const;

 private:
  Duration target_cycle_time_;
  ImageViewer viewer_;
  FPSEstimator fps_;
  TimePoint current_frame_finish_;
  TimePoint next_frame_finish_;
};
