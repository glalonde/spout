#include "base/init.h"
#include "vulkan/image_viewer/image_viewer.h"

int main(int argc, char* argv[]) {
  Init(argc, argv);
  ImageViewerApplication app;
  app.Run();
  return EXIT_SUCCESS;
}
