#include "src/fps_estimator.h"
#include <cmath>

FPSEstimator::FPSEstimator(Duration window, double estimated_max_frequency)
    : deltas_(InitBuffer(window, estimated_max_frequency)), sum_(0), current_estimate_(0) {}

double FPSEstimator::CurrentEstimate() const {
  return current_estimate_;
}

double FPSEstimator::Tick(Duration delta) {
  sum_ += delta;
  sum_ -= *deltas_.NextOverwritten();
  deltas_.Push(delta);
  current_estimate_ = deltas_.data().size() / ToSeconds<double>(sum_);
  return current_estimate_;
}

// Create a fixed size buffer to average FPS over an estimated duration.
CircularBuffer<Duration> FPSEstimator::InitBuffer(
    Duration window, double estimated_max_frequency) {
  const int estimated_number = static_cast<int>(
      std::ceil(ToSeconds<double>(window) * estimated_max_frequency));
  return CircularBuffer<Duration>(estimated_number, Duration(0));
}
