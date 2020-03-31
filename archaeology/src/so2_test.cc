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

  EXPECT_NEAR(r3.radians(), 0.0, std::numeric_limits<double>::epsilon());

  SO2d r4(M_PI / 2.0);
  EXPECT_NEAR(r4.radians(), M_PI / 2.0, std::numeric_limits<double>::epsilon());
  SO2d r5 = r4.inverse();
  EXPECT_NEAR(r5.radians(), -M_PI / 2.0,
              std::numeric_limits<double>::epsilon());
  EXPECT_TRUE(r1.is_normalized());
  EXPECT_TRUE(r2.is_normalized());
  EXPECT_TRUE(r3.is_normalized());
  EXPECT_TRUE(r4.is_normalized());
  EXPECT_TRUE(r5.is_normalized());

  SO2d r6;
  auto* coeffs = const_cast<Vector2d*>(&r6.data());
  *coeffs = Vector2d(2.0, 1.0);
  EXPECT_FALSE(r6.is_normalized());
  r6.Normalize();
  EXPECT_TRUE(r6.is_normalized());
  EXPECT_NEAR(r6.radians(), std::atan(1.0 / 2.0),
              std::numeric_limits<double>::epsilon());
}

GTEST_TEST(SO2Test, Fixed) {
  SO2d r1(0);
  EXPECT_DOUBLE_EQ(CWRotate90(r1).radians(), -M_PI / 2.0);
  EXPECT_DOUBLE_EQ(CCWRotate90(r1).radians(), M_PI / 2.0);
  EXPECT_DOUBLE_EQ(Rotate180(SO2d(M_PI / 2.0)).radians(), -M_PI / 2.0);
}

GTEST_MAIN();
