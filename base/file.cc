#include "base/file.h"

#include <cstring>
#include <fstream>
#include <system_error>

#include "base/logging.h"

std::string ReadFileOrDie(const std::string& filepath) {
  std::ifstream file;
  file.open(filepath, std::ifstream::in);
  CHECK(file.good()) << "Reading: '" << filepath
                     << "' failed: " << strerror(errno);
  return std::string(std::istreambuf_iterator<char>(file),
                     std::istreambuf_iterator<char>());
}
