#include "src/int_grid.h"
#include <bitset>
#include "base/googletest.h"

GTEST_TEST(IntGrid, Smoke) {
  uint32_t v = 255;
  EXPECT_EQ(GetHighRes<8>(v), 255);
  EXPECT_EQ(GetLowRes<8>(v), 0);
  v = 256;
  EXPECT_EQ(GetHighRes<8>(v), 0);
  EXPECT_EQ(GetLowRes<8>(v), 1);
  std::cout << std::bitset<32>(kHighResMask<8>) << std::endl;
}

GTEST_TEST(IntGrid, Anchor) {
  std::cout << std::bitset<32>(kAnchor<uint32_t, 8>) << std::endl;
  LOG(INFO) << kAnchor<uint32_t, 8>;
}

GTEST_MAIN();
