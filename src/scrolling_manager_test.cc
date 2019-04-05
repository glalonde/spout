#include "src/scrolling_manager.h"
#include "base/googletest.h"

GTEST_TEST(ScrollingManager, Smoke) {
  ScrollingManager man(100 /* buffer size */, 200 /* viewport size */);
  EXPECT_EQ(man.lowest_visible_buffer(), 0);
  EXPECT_EQ(man.highest_visible_buffer(), 1);
  man.UpdateHeight(1);
  EXPECT_EQ(man.lowest_visible_buffer(), 0);
  EXPECT_EQ(man.highest_visible_buffer(), 2);
  man.UpdateHeight(0);
  EXPECT_EQ(man.lowest_visible_buffer(), 0);
  EXPECT_EQ(man.highest_visible_buffer(), 1);

  // Still seeing the top row of the lowest buffer
  man.UpdateHeight(99);
  EXPECT_EQ(man.lowest_visible_buffer(), 0);
  EXPECT_EQ(man.highest_visible_buffer(), 2);

  // Lowest buffer no longer visible
  man.UpdateHeight(100);
  EXPECT_EQ(man.lowest_visible_buffer(), 1);
  EXPECT_EQ(man.highest_visible_buffer(), 2);

  // Can see 4th buffer for first time
  man.UpdateHeight(101);
  EXPECT_EQ(man.lowest_visible_buffer(), 1);
  EXPECT_EQ(man.highest_visible_buffer(), 3);
}

GTEST_MAIN();
