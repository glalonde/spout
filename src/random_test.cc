#include "src/random.h"
#include <Eigen/Core>
#include "base/googletest.h"

GTEST_TEST(EigenTest, SetRandomDistribution) {
  std::mt19937 gen(0);
  std::uniform_real_distribution<float> dis1(.3, .6);
  MatrixXf test(10, 10);
  test.setConstant(6);
  SetRandomDistribution(&dis1, &gen, test.row(5));
  for (int col = 0; col < test.cols(); ++col) {
    EXPECT_GE(test(5, col), .3);
    EXPECT_LE(test(5, col), .6);
  }
  std::uniform_real_distribution<float> dis2(-5, -10);
  SetRandomDistribution(&dis2, &gen, test.col(5));
  for (int row = 0; row < test.rows(); ++row) {
    EXPECT_GE(test(row, 5), -10);
    EXPECT_LE(test(row, 5), -5);
  }
}

GTEST_TEST(EigenTest, SetRandomUniform) {
  std::mt19937 gen(0);
  MatrixXf test(10, 10);
  test.setConstant(6);
  SetRandomUniform(.3, .5, &gen, test.row(5));
  for (int col = 0; col < test.cols(); ++col) {
    EXPECT_GE(test(5, col), .3);
    EXPECT_LE(test(5, col), .6);
  }
  SetRandomUniform(-10, -5, &gen, test.col(5));
  for (int row = 0; row < test.rows(); ++row) {
    EXPECT_GE(test(row, 5), -10);
    EXPECT_LE(test(row, 5), -5);
  }
}

GTEST_TEST(EigenTest, SetRandomUniformInt) {
  std::mt19937 gen(0);
  MatrixXi test(10, 10);
  test.setConstant(6);
  SetRandomUniform(15, 35, &gen, test.row(5));
  for (int col = 0; col < test.cols(); ++col) {
    EXPECT_GE(test(5, col), 15);
    EXPECT_LE(test(5, col), 35);
  }
  SetRandomUniform(-10, -5, &gen, test.col(5));
  for (int row = 0; row < test.rows(); ++row) {
    EXPECT_GE(test(row, 5), -10);
    EXPECT_LE(test(row, 5), -5);
  }
}

GTEST_MAIN();
