#include "src/eigen_types.h"
#include "base/googletest.h"

GTEST_TEST(MoveTest, Smoke) {
  MatrixXd mat = MatrixXd::Random(3, 2);
  EXPECT_EQ(mat.size(), 6);
  MatrixXd mat_moved = std::move(mat);
  EXPECT_EQ(mat.size(), 0);
  EXPECT_EQ(mat_moved.size(), 6);
}

Vector2i DivideDownward(const Vector2i& hr, int cell_size) {
  const Vector2i signs = hr.cwiseSign();
  hr / cell_size(hr.cwiseAbs() / cell_size)

              LOG(INFO)
      << hr.cwiseAbs().transpose();
  LOG(INFO) << (hr.cwiseAbs() / cell_size).transpose();
  return (hr.cwiseAbs() / cell_size).cwiseProduct(hr.cwiseSign());
}

GTEST_TEST(MoveTest, Integral) {
  Vector2i hr(-5, -15);
  const int kCellSize = 10;
  Vector2i lr(-1, -2);
  LOG(INFO) << DivideDownward(hr, kCellSize).transpose();
}

GTEST_MAIN();
