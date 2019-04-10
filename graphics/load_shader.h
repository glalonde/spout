#pragma once
#include <string>
#include "graphics/opengl.h"

GLuint LoadShader(const std::string& shader_path, GLuint shader_type);

void LinkProgram(GLuint program);
