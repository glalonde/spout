#include "base/googletest.h"

GTEST_TEST(LogTest, Smoke) {
  LOG(INFO) << "Hello World.";
  LOG(WARNING) << "Hello World.";
  LOG(ERROR) << "Hello World.";
  CHECK_EQ(4, 2 + 2);
}

GTEST_TEST(LogTest, Fatal) {
  EXPECT_DEATH(LOG(FATAL) << "yo", "yo");
}

GTEST_TEST(LogTest, Check) {
  EXPECT_DEATH(CHECK_EQ(1, 2) << "Not equal", "Not equal");
}

GTEST_MAIN()
