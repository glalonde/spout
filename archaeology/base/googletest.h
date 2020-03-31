#pragma once

#include <gtest/gtest.h>
#include <cstring>
#include "base/init.h"
#include "base/logging.h"

// Define "main()" for a unit test using gtest.
static bool kIsUnitTest = false;

void CheckEnv(char** envp) {
  static const std::string kUnitTestFlag = "BAZEL_UNIT_TEST=1";
  for (char** env = envp; *env != 0; env++) {
    if (std::strcmp(*env, kUnitTestFlag.c_str()) == 0) {
      kIsUnitTest = true;
      break;
    }
  }
}

#define GTEST_MAIN()                             \
  int main(int argc, char** argv, char** envp) { \
    CheckEnv(envp);                              \
    testing::InitGoogleTest(&argc, argv);        \
    Init(argc, argv);                            \
    return RUN_ALL_TESTS();                      \
  }
