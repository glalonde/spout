#include "src/fps_estimator.h"
#include <cmath>

FPSEstimator::FPSEstimator(Duration window, double target_frequency)
    : target_(FromHz(target_frequency)),
      deltas_(InitBuffer(window, target_frequency)),
      sum_(0),
      current_estimate_(0),
      previous_(Now()) {}

double FPSEstimator::CurrentEstimate() const {
  return current_estimate_;
}

Duration FPSEstimator::Tick() {
  // Time elapsed since the end of the last cycle.
  Duration initial_delta = Now() - previous_;
  if (target_ > initial_delta) {
    HighResSleepFor(target_ - initial_delta);
  }
  // "End" of the previous cycle. We want a target period between these calls.
  TimePoint end_time = Now();
  Duration final_delta = end_time - previous_;
  UpdateEstimate(final_delta);
  previous_ = end_time;
  return final_delta;
}

void FPSEstimator::UpdateEstimate(Duration delta) {
  sum_ += delta;
  sum_ -= *deltas_.NextOverwritten();
  deltas_.Push(delta);
  current_estimate_ = deltas_.data().size() / ToSeconds<double>(sum_);
}

// Create a fixed size buffer to average FPS over an estimated duration.
CircularBuffer<Duration> FPSEstimator::InitBuffer(
    Duration window, double estimated_max_frequency) {
  const int estimated_number = static_cast<int>(
      std::ceil(ToSeconds<double>(window) * estimated_max_frequency));
  return CircularBuffer<Duration>(estimated_number, Duration(0));
}
