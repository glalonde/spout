#pragma once
#include <gperftools/profiler.h>
#include "base/construction_macros.h"
#include "base/logging.h"

DEFINE_string(profile_output, "", "Output path for profiler file");

class ScopedProfiler {
 public:
  NO_COPY_NO_MOVE_NO_ASSIGN(ScopedProfiler);

  ScopedProfiler(const std::string& profile_output) : started_(false) {
    if (profile_output.empty()) {
      LOG(ERROR) << "No output path specified, not starting profiler";
    } else {
      if (ProfilerStart(profile_output.c_str()) > 0) {
        LOG(INFO) << "Writing profiler output to: " << profile_output;
        started_ = true;
      }
    }
  }

  ~ScopedProfiler() {
    if (started_) {
      ProfilerStop();
      ProfilerFlush();
    }
  }

 private:
  bool started_;
};
