#pragma once
#include <chrono>
#include <iostream>
#include <thread>
#include "base/logging.h"

using ClockType = std::chrono::high_resolution_clock;
using TimePoint = ClockType::time_point;
using Duration = ClockType::duration;

// Converts duration to number seconds. Will return fractional seconds if T is
// a floating point type.
//
// For example: ToSeconds<double>(std::chrono::milliseconds(10)) == 0.01
template <class T>
inline T ToSeconds(Duration duration) {
  return std::chrono::duration<T, std::chrono::seconds::period>(duration)
      .count();
}

// Converts a time point to the number of seconds since the epoch of its
// corresponding clock.
template <class T>
inline T ToSecondsSinceEpoch(TimePoint time_point) {
  return ToSeconds<T>(time_point - TimePoint());
}

// Converts a number of seconds to a duration.
//
// For example:
//   FromSeconds(1) == std::chrono::seconds(1)
//   FromSeconds(0.5) == std::chrono::milliseconds(500)
//   FromSeconds(0.001) == std::chrono::milliseconds(1)
template <class T>
constexpr inline Duration FromSeconds(T seconds) {
  return std::chrono::duration_cast<Duration>(
      std::chrono::duration<T, std::chrono::seconds::period>(seconds));
}

// Converts a number of seconds since epoch to a time point.
template <class T>
inline TimePoint FromSecondsSinceEpoch(T seconds) {
  return TimePoint() + FromSeconds(seconds);
}

// Converts a number of cycles per second to a duration per cycle.
//
// For example:
//   FromHz(1.0) == std::chrono::seconds(1)
//   FromHz(100) == std::chrono::milliseconds(10)
constexpr inline Duration FromHz(double hz) {
  return FromSeconds(1.0 / hz);
}

// Turns a Duration period into hertz.
inline double ToHz(Duration period) {
  return 1.0 / ToSeconds<double>(period);
}

// Uses native sleep until the accuracy limit, then busy waits.
inline void HighResSleepFor(Duration d) {
  const auto start = std::chrono::high_resolution_clock::now();
  const auto end = start + d;
  static constexpr Duration kNativeAccuracy = FromSeconds<double>(0.0005);
  const Duration remainder = d % kNativeAccuracy;
  std::this_thread::sleep_for(d - remainder);
  while (std::chrono::high_resolution_clock::now() < end) {
  }
}

// String formatting
namespace std {
std::ostream& operator<<(std::ostream& os, const TimePoint& time_point);
std::ostream& operator<<(std::ostream& os, const Duration& duration);
}  // namespace std
