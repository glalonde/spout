#include "base/file.h"
#include "base/init.h"
#include "src/image_io.h"
#include "graphics/image_viewer.h"

ABSL_FLAG(std::string, image_path, "", "Image path");

int main(int argc, char* argv[]) {
  Init(argc, argv);
  auto maybe_image = ReadImage(absl::GetFlag(FLAGS_image_path));
  CHECK(maybe_image) << "Couldn't load image at: "
                     << absl::GetFlag(FLAGS_image_path);
  ImageViewer viewer(maybe_image->cols(), maybe_image->rows());
  auto* data = viewer.data();
  *data = *maybe_image;
  while (!viewer.Update().quit) {
  }
  return 0;
}
