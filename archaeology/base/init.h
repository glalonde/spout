#pragma once
#include "base/logging.h"

void Init(int& argc, char** argv) {
  // Init glog
  google::InitGoogleLogging(argv[0]);
  google::InstallFailureSignalHandler();
  absl::ParseCommandLine(argc, argv);
  // Use console output for logging rather than file.
  FLAGS_logtostderr = true;
}
