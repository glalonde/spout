#pragma once
#include "base/logging.h"
#include "graphics/opengl.h"

inline bool CheckGLErrors() {
  GLenum e = glGetError();
  if (e != GL_NO_ERROR) {
    LOG(ERROR) << "OpenGL Error: " << e << ", " << gluErrorString(e);
    return false;
  }
  return true;
}
