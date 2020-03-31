#include <optional>
#include "base/googletest.h"

GTEST_TEST(Optional, Smoke) {
  std::optional<int> x;
  EXPECT_FALSE(x);
  x = 1;
  ASSERT_TRUE(x);
  EXPECT_EQ(*x, 1);
  x = {};
  EXPECT_FALSE(x);
}

GTEST_MAIN()
