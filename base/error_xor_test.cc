#include "base/error_xor.h"
#include "base/googletest.h"

GTEST_TEST(ErrorXor, Smoke) {
  ErrorXor<std::string> maybe_string;
  EXPECT_FALSE(maybe_string);
  maybe_string = std::string("asdf");
  EXPECT_TRUE(maybe_string);
  EXPECT_EQ(maybe_string.ErrorOrNull(), nullptr);
}

GTEST_TEST(ErrorXor, Void) {
  ErrorXor<void> maybe_error;
  EXPECT_FALSE(maybe_error);
  const std::string msg = "This is an error message";
  maybe_error = ErrorXor<void>(ErrorMessage(msg));
  EXPECT_EQ(msg, maybe_error.ErrorOrDie().message());
  EXPECT_FALSE(maybe_error);
  LOG(INFO) << maybe_error;
  maybe_error = ErrorXor<void>::NoError();
  EXPECT_TRUE(maybe_error);
  LOG(INFO) << maybe_error;
}

GTEST_TEST(ErrorXor, TraceError) {
  ErrorXor<int> maybe_int;
  EXPECT_FALSE(maybe_int);
  LOG(INFO) << maybe_int;
  maybe_int = ErrorXor<int>(TraceError("Couldn't find int."));
  EXPECT_FALSE(maybe_int);
  LOG(INFO) << maybe_int;
}

GTEST_MAIN()
