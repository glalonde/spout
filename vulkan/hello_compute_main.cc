#include "base/init.h"
#include "vulkan/hello_compute.h"

// Currently these have to match the constants defined in the shader
// mandelbrot.comp
// TODO(glalonde) convert those to uniforms
ABSL_FLAG(int32_t, width, 3200, "Image width");
ABSL_FLAG(int32_t, height, 2400, "Image height");
ABSL_FLAG(std::string, out_path, "", "Image output path");

int main(int argc, char* argv[]) {
  Init(argc, argv);
  ComputeApplication app;
  const std::string output = absl::GetFlag(FLAGS_out_path);
  app.Run(absl::GetFlag(FLAGS_width), absl::GetFlag(FLAGS_height), output);
  return EXIT_SUCCESS;
}
