#pragma once

#include <gtest/gtest.h>
#include "base/init.h"
#include "base/logging.h"

// Define "main()" for a unit test using gtest.
#define GTEST_MAIN()                      \
  int main(int argc, char** argv) {       \
    testing::InitGoogleTest(&argc, argv); \
    Init(argc, argv);                     \
    return RUN_ALL_TESTS();               \
  }
