#pragma once
#include <memory>
#include "src/controller_input.h"
#include "src/image.h"

class ParticlesDemo {
 public:
  ParticlesDemo(int window_width, int window_height);
  void SetTextureSize(int width, int height);
  void SetWindowSize(int width, int height);
  ~ParticlesDemo();
  ControllerInput Update();
  void SetDataChanged();

  bool IsFullScreen();
  void ToggleFullScreen();

 private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};
