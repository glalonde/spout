#include "src/image_io.h"
#include "base/googletest.h"

GTEST_TEST(ImageSmoke, Load) {
  ReadImage("src/testdata/test.png");
}

GTEST_MAIN();
