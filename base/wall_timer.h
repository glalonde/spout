#pragma once
#include "base/time.h"

// A simple timer which measures wall-clock (i.e. real-world) time.
class WallTimer {
 public:
  using WallClock = std::chrono::high_resolution_clock;

  WallTimer() : started_(false), stopped_(false) {}

  ~WallTimer() {}

  // Starts the timer.
  void Start() {
    started_ = true;
    stopped_ = false;
    start_time_ = WallClock::now();
  }

  // Stops the timer.
  void Stop() {
    stop_time_ = WallClock::now();
    stopped_ = true;
  }

  // Returns the duration elapsed since Start() was called.
  Duration ElapsedDuration() const {
    if (!started_) {
      // Timer never started, return zero.
      return FromSeconds(0);
    } else if (!stopped_) {
      // Timer started but not stopped yet, return time from start to now.
      return WallClock::now() - start_time_;
    } else {
      // Timer started and stopped, return time elapsed.
      return stop_time_ - start_time_;
    }
  }

  // Returns the number of seconds elapsed since Start() was called.
  double ElapsedSeconds() const {
    return ToSeconds<double>(ElapsedDuration());
  }

 private:
  bool started_;
  bool stopped_;
  WallClock::time_point start_time_;
  WallClock::time_point stop_time_;
};
