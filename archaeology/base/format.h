#pragma once

#include <tinyformat.h>

template <class T, class... Args>
std::string FormatString(const T& str, Args&&... args) {
  return tinyformat::format(str.c_str(), std::forward<Args>(args)...);
}

template <class... Args>
std::string FormatString(const char* str, Args&&... args) {
  return tinyformat::format(str, std::forward<Args>(args)...);
}
