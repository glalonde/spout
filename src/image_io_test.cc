#include "src/image_io.h"
#include "base/googletest.h"

GTEST_TEST(ImageSmoke, Load) {
  ReadImage("src/testdata/test.png");
}

GTEST_TEST(ImageSmoke, Write) {
  const std::string name = "/tmp/io_test.png";
  auto maybe_image = ReadImage("src/testdata/test.png");
  ASSERT_TRUE(maybe_image);
  WriteImage(*maybe_image, name);
  LOG(INFO) << "Maybe wrote image to: " << name;
}

GTEST_MAIN();
