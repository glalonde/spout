#pragma once
#include <string>
#include "base/error_xor.h"

ErrorXor<std::string> TryReadFile(const std::string& path);
std::string ReadFileOrDie(const std::string& path);

// TODO(glalonde) make ErrorXor<void> work, use it here
std::optional<ErrorMessage> TryWriteFile(const std::string& path,
                                         const std::string& data);
void WriteFileOrDie(const std::string& path, const std::string& data);
