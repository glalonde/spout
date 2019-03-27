#include "src/convert.h"
#include "base/googletest.h"

GTEST_TEST(Convert, Smoke1) {
  PixelType::RGBU8 p(252, 253, 254);
  PixelType::RGBAU8 pc = Convert<PixelType::RGBAU8>(p);
  EXPECT_EQ(pc, PixelType::RGBAU8(252, 253, 254, 255));
}

GTEST_TEST(Convert, Smoke2) {
  PixelType::RGBAU8 p(252, 253, 254, 255);
  PixelType::RGBU8 pc = Convert<PixelType::RGBU8>(p);
  EXPECT_EQ(pc, PixelType::RGBU8(252, 253, 254));
}

GTEST_TEST(Convert, Smoke3) {
  PixelType::RGBF32 p(0.0, .9999, 1.0);
  PixelType::RGBU8 pc = Convert<PixelType::RGBU8>(p);
  EXPECT_EQ(pc, PixelType::RGBU8(0, 255, 255));
}

GTEST_TEST(Convert, Smoke4) {
  PixelType::RGBF32 p(0.0, .9999, 1.0);
  PixelType::RGBAU8 pc = Convert<PixelType::RGBAU8>(p);
  EXPECT_EQ(pc, PixelType::RGBAU8(0, 255, 255, 255));
}

GTEST_MAIN();
