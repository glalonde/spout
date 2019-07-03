#include "base/init.h"
#include "vulkan/hello_compute.h"

ABSL_FLAG(int32_t, width, 1000, "Image width");
ABSL_FLAG(int32_t, height, 1000, "Image height");

int main(int argc, char* argv[]) {
  Init(argc, argv);
  ComputeApplication app;
  app.Run(absl::GetFlag(FLAGS_width), absl::GetFlag(FLAGS_height));
  return EXIT_SUCCESS;
}
