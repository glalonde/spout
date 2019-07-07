#pragma once
#include <optional>
#include <string>

#include "src/image.h"

std::optional<Image<PixelType::RGBAU8>> ReadImage(const std::string& path);
void WriteImage(const Image<PixelType::RGBAU8>& image,
                const std::string& path);
