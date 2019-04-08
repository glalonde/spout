#include "src/buffer_stack.h"
#include "base/googletest.h"

GTEST_TEST(MoveTest, Smoke) {
  const int kRowsPerBuffer = 10;
  BufferStack<MatrixXi> buffers(kRowsPerBuffer);

  // Moves when possible
  for (int i = 0; i < 10; ++i) {
    MatrixXi mat = MatrixXi::Random(kRowsPerBuffer, 2);
    buffers.EmplaceBuffer(std::move(mat));
    EXPECT_EQ(mat.size(), 0);
  }

  // Otherwise copies.
  for (int i = 0; i < 10; ++i) {
    MatrixXi mat = MatrixXi::Random(kRowsPerBuffer, 2);
    buffers.EmplaceBuffer(mat);
    EXPECT_EQ(mat.size(), 20);
  }
}

GTEST_MAIN();
