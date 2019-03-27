#pragma once
#include "base/logging.h"

void Init(int& argc, char** argv) {
  // Init glog
  google::InitGoogleLogging(argv[0]);
  google::InstallFailureSignalHandler();
  // Parse cmd line flags
  gflags::ParseCommandLineFlags(&argc, &argv, true);
  // Use console output for logging rather than file.
  FLAGS_logtostderr = true;
}
