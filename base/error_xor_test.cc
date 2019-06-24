#include "base/error_xor.h"
#include "base/googletest.h"

GTEST_TEST(ErrorXor, Smoke) {
  ErrorXor<std::string> maybe_string;
  EXPECT_FALSE(maybe_string);

  maybe_string = std::string("asdf");
  EXPECT_TRUE(maybe_string);
  EXPECT_EQ(maybe_string.ErrorOrNull(), nullptr);
}

GTEST_MAIN()
