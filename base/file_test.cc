#include "base/file.h"
#include "base/googletest.h"

#include <cstdio>

GTEST_TEST(File, Smoke) {
  const std::string name = std::tmpnam(nullptr);
  const std::string value = "ayyyyyooooo";
  auto maybe_error = TryWriteFile(name, value);
  ASSERT_FALSE(maybe_error);

  auto maybe_file = TryReadFile(name);
  ASSERT_TRUE(maybe_file);
  EXPECT_EQ(*maybe_file.ValueOrNull(), value);
}



GTEST_TEST(File, SmokeFail) {
  const std::string doesnt_exist = "??";
  auto maybe_file = TryReadFile(doesnt_exist);
  ASSERT_FALSE(maybe_file);
  LOG(INFO) << maybe_file.ErrorOrDie();
}

GTEST_MAIN()
