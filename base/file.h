#pragma once
#include <string>
#include "base/error_xor.h"

ErrorXor<std::string> TryReadFile(const std::string_view& path);
std::string ReadFileOrDie(const std::string_view& path);

// TODO(glalonde) make ErrorXor<void> work, use it here
std::optional<ErrorMessage> TryWriteFile(const std::string_view& path,
                                         const std::string_view& data);
void WriteFileOrDie(const std::string_view& path, const std::string_view& data);
