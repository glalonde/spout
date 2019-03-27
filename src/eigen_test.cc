#include <Eigen/Core>
#include "base/googletest.h"
#include "src/eigen_types.h"

GTEST_TEST(EigenTest, SetRandomRange) {
  MatrixXf test(10, 10);
  test.setConstant(6);
  SetRandomRange(.3, .6, test.row(5));
  for (int col = 0; col < test.cols(); ++col) {
    EXPECT_GE(test(5, col), .3);
    EXPECT_LE(test(5, col), .6);
  }
  SetRandomRange(-5, -10, test.col(5));
  for (int row = 0; row < test.rows(); ++row) {
    EXPECT_GE(test(row, 5), -10);
    EXPECT_LE(test(row, 5), -5);
  }
}

GTEST_MAIN();
