#include "base/format.h"
#include "base/googletest.h"

GTEST_TEST(Format, Smoke) {
  const std::string string = FormatString("format_%s", 5);
  EXPECT_EQ(string, "format_5");
}

GTEST_MAIN()
