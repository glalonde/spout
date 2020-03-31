#include "base/file.h"

#include <cstring>
#include <fstream>
#include <system_error>

#include "base/logging.h"
#include "base/format.h"

ErrorXor<std::string> TryReadFile(const std::string& path) {
  std::ifstream file;
  file.open(path, std::ifstream::in);
  if (!file.good()) {
    return ErrorMessage(
        FormatString("Reading: %s failed: %s", path, strerror(errno)));
  }
  return std::string(std::istreambuf_iterator<char>(file),
                     std::istreambuf_iterator<char>());
}

std::string ReadFileOrDie(const std::string& path) {
  auto maybe_file = TryReadFile(path);
  CHECK(maybe_file) << *maybe_file.ErrorOrNull();
  return std::move(*maybe_file.ValueOrNull());
}

std::optional<ErrorMessage> TryWriteFile(const std::string& path,
                                         const std::string& data) {
  std::ofstream out(path);
  if (out) {
    out << data;
    return {};
  } else {
    return ErrorMessage(
        FormatString("Reading: %s failed: %s", path, strerror(errno)));
  }
}

void WriteFileOrDie(const std::string& path,
                    const std::string& data) {
  auto maybe_error = TryWriteFile(path, data);
  CHECK(!maybe_error) << *maybe_error;
}
