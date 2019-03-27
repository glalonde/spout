#pragma once

#include "src/controller_input.h"
#include "src/image.h"

class ImageViewer {
 public:
  ImageViewer(int width, int height);
  ~ImageViewer();
  Image<PixelType::RGBAU8>* data();
  ControllerInput Update();

 private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};
