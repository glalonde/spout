#include "base/circular_buffer.h"
#include "base/googletest.h"

GTEST_TEST(CircularBuffer, Smoke) {
  CircularBuffer<int> dubs(10);
  for (int i = 0; i < dubs.Capacity(); ++i) {
    EXPECT_EQ(dubs.NextOverwritten(), nullptr);
    dubs.Push(i);
  }
  EXPECT_EQ(dubs.NextOverwritten(), &dubs.data()[0]);
  EXPECT_EQ(dubs.WriteIndex(), 0);
  dubs.Push(10);
  EXPECT_EQ(dubs.data()[0], 10);
  for (int i = 1; i < 10; ++i) {
    EXPECT_EQ(dubs.data()[i], i);
  }
}

GTEST_MAIN()
