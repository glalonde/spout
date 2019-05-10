#pragma once
#include <string>
#include <vector>
#include "graphics/opengl.h"

GLuint LoadShader(const std::string& shader_path, GLuint shader_type);
GLuint LoadShader(const std::vector<std::string>& shader_path,
                  GLuint shader_type);

void LinkProgram(GLuint program);
