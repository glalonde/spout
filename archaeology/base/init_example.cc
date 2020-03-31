#include "base/init.h"

ABSL_FLAG(int32_t, flag_num, -1, "Flag number");

int main(int argc, char** argv) {
  Init(argc, argv);
  LOG(INFO) << "Flag num: " << absl::GetFlag(FLAGS_flag_num);
  return 0;
}
