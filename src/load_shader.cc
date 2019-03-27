#include "src/load_shader.h"
#include "base/file.h"
#include "base/logging.h"

GLuint LoadShader(const std::string& shader_path, GLuint shader_type) {
  const std::string shader_source = ReadFileOrDie(shader_path);
  const auto* source_ptr = shader_source.c_str();
  int source_length = shader_source.size();
  GLuint shader_handle = glCreateShader(shader_type);
  glShaderSource(shader_handle, 1, &source_ptr, &source_length);
  glCompileShader(shader_handle);
  int status;
  glGetShaderiv(shader_handle, GL_COMPILE_STATUS, &status);
  if (status == GL_FALSE) {
    GLint length;
    glGetShaderiv(shader_handle, GL_INFO_LOG_LENGTH, &length);
    if (length <= 0) {
      LOG(FATAL) << "Couldn't read shader compiler log length.";
    }
    std::string log(length - 1, '0');
    glGetShaderInfoLog(shader_handle, log.size(), NULL, log.data());
    LOG(FATAL) << "Couldn't load shader: " << std::endl << log;
  }
  return shader_handle;
}

void LinkProgram(GLuint program) {
  glLinkProgram(program);
  int status;
  glGetProgramiv(program, GL_LINK_STATUS, &status);
  if (status == GL_FALSE) {
    GLint length;
    glGetProgramiv(program, GL_INFO_LOG_LENGTH, &length);
    if (length <= 0) {
      LOG(FATAL) << "Couldn't read program linker log length.";
    }
    std::string log(length - 1, '0');
    glGetProgramInfoLog(program, log.size(), NULL, log.data());
    LOG(FATAL) << "Couldn't link program: " << std::endl << log;
  }
}
