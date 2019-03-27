#pragma once
#include <optional>
#include <string>

#include "src/image.h"

std::optional<Image<PixelType::RGBAU8>> ReadImage(const std::string& path);
