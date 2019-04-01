#pragma once
#include "base/circular_buffer.h"
#include "base/time.h"

// Estimate frames per second.
class FPSEstimator {
 public:
  // Keep track of the average FPS over the last `window` of updates, at a
  // target frequency.
  FPSEstimator(Duration window, double estimated_max_frequency);

  // Update the estimator: Give it a cycle time, and it returns the current
  // average FPS.
  double Tick(Duration delta);

 private:
  // Create a fixed size buffer to average FPS over an estimated duration.
  static CircularBuffer<Duration> InitBuffer(Duration window,
                                             double estimated_max_frequency);

  // Store the last N cycle times
  CircularBuffer<Duration> deltas_;
  Duration sum_;
};
