#pragma once

#include "src/image.h"

class ImageViewer {
 public:
  ImageViewer(Image<PixelType::RGBAU8> image);
  ~ImageViewer();
  void Loop();

 private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};
