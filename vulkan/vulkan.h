#pragma once
#include <vulkan/vulkan.h>

#ifdef NDEBUG
static constexpr bool kVulkanDebugMode = false;
#else
static constexpr bool kVulkanDebugMode = true;
#endif
