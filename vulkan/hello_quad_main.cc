#include "base/init.h"
#include "vulkan/hello_quad.h"

int main(int argc, char* argv[]) {
  Init(argc, argv);
  HelloQuadApplication app;
  app.Run();
  return EXIT_SUCCESS;
}
