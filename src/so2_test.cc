#include "src/so2.h"
#include "base/googletest.h"

GTEST_TEST(SO2Test, Smoke) {
  SO2d r1(M_PI);
  EXPECT_NEAR(r1.cos(), -1.0, std::numeric_limits<double>::epsilon());
  EXPECT_NEAR(r1.sin(), 0.0, std::numeric_limits<double>::epsilon());

  SO2d r2(-M_PI);
  EXPECT_NEAR(r2.cos(), -1.0, std::numeric_limits<double>::epsilon());
  EXPECT_NEAR(r2.sin(), 0.0, std::numeric_limits<double>::epsilon());

  SO2d r3 = r1 * r2;
  EXPECT_NEAR(r3.cos(), 1.0, std::numeric_limits<double>::epsilon());
  EXPECT_NEAR(r3.sin(), 0.0, std::numeric_limits<double>::epsilon());
}

GTEST_MAIN();
