#include "src/eigen_types.h"
#include "base/googletest.h"

GTEST_TEST(MoveTest, Smoke) {
  MatrixXd mat = MatrixXd::Random(3, 2);
  EXPECT_EQ(mat.size(), 6);
  MatrixXd mat_moved = std::move(mat);
  EXPECT_EQ(mat.size(), 0);
  EXPECT_EQ(mat_moved.size(), 6);
}

GTEST_MAIN();
