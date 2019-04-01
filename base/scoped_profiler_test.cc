#include "base/scoped_profiler.h"
#include <cstdio>
#include <thread>
#include "base/file.h"
#include "base/googletest.h"
#include "base/time.h"

GTEST_TEST(ScopedProfiler, Smoke) {
  const std::string name = std::tmpnam(nullptr);
  {
    ScopedProfiler prof(name);
    std::this_thread::sleep_for(FromSeconds<double>(2.0));
  }
  auto output = ReadFileOrDie(name);
  EXPECT_FALSE(output.empty());
}

GTEST_MAIN()
