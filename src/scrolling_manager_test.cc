#include "src/scrolling_manager.h"
#include "base/googletest.h"

GTEST_TEST(ScrollingManager, Smoke) {
  ScrollingManager man(100 /* buffer size */, 200 /* viewport size */);
  const auto& visible_buffers = man.visible_buffers();
  ASSERT_EQ(man.num_visible_buffers(), 2);
  EXPECT_EQ(visible_buffers[0], 0);
  EXPECT_EQ(visible_buffers[1], 1);
  man.UpdateHeight(1);
  ASSERT_EQ(man.num_visible_buffers(), 3);
  EXPECT_EQ(visible_buffers[0], 0);
  EXPECT_EQ(visible_buffers[1], 1);
  EXPECT_EQ(visible_buffers[2], 2);
  man.UpdateHeight(0);
  ASSERT_EQ(man.num_visible_buffers(), 2);
  EXPECT_EQ(visible_buffers[0], 0);
  EXPECT_EQ(visible_buffers[1], 1);

  // Still seeing the top row of the lowest buffer
  man.UpdateHeight(99);
  ASSERT_EQ(man.num_visible_buffers(), 3);
  EXPECT_EQ(visible_buffers[0], 0);
  EXPECT_EQ(visible_buffers[1], 1);
  EXPECT_EQ(visible_buffers[2], 2);

  // Lowest buffer no longer visible
  man.UpdateHeight(100);
  ASSERT_EQ(man.num_visible_buffers(), 2);
  EXPECT_EQ(visible_buffers[0], 1);
  EXPECT_EQ(visible_buffers[1], 2);

  // Can see 4th buffer for first time
  man.UpdateHeight(101);
  ASSERT_EQ(man.num_visible_buffers(), 3);
  EXPECT_EQ(visible_buffers[0], 1);
  EXPECT_EQ(visible_buffers[1], 2);
  EXPECT_EQ(visible_buffers[2], 3);
}

GTEST_MAIN();
