#include "base/time.h"
#include <thread>
#include "base/googletest.h"
#include "base/wall_timer.h"

void NativeSleepFor(Duration d) {
  std::this_thread::sleep_for(d);
}

GTEST_TEST(Sleep, Smoke) {
  const Duration period = FromHz(60.0);
  WallTimer timer;
  double max_d;
  for (int i = 0; i < 100; ++i) {
    timer.Start();
    NativeSleepFor(period);
    timer.Stop();
    Duration error = timer.ElapsedDuration() - period;
    max_d = std::max(max_d, std::abs(ToSeconds<double>(error)));
  }
  LOG(INFO) << "std::this_thread::sleep_for error: " << max_d;
}

GTEST_TEST(Sleep, Smoke2) {
  const Duration period = FromHz(60.0);
  WallTimer timer;
  double max_d;
  for (int i = 0; i < 100; ++i) {
    timer.Start();
    HighResSleepFor(period);
    timer.Stop();
    Duration error = timer.ElapsedDuration() - period;
    max_d = std::max(max_d, std::abs(ToSeconds<double>(error)));
  }
  LOG(INFO) << "TestSleepFor error: " << max_d;
}

GTEST_MAIN()
