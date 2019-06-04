#pragma once
#include <vulkan/vulkan.h>

// Debug messenger requires extension `VK_EXT_DEBUG_UTILS_EXTENSION_NAME`
class VulkanDebugMessenger {
 public:
  VulkanDebugMessenger(VkInstance instance);
  ~VulkanDebugMessenger();

 private:
  static VKAPI_ATTR VkBool32 VKAPI_CALL DebugCallback(
      VkDebugUtilsMessageSeverityFlagBitsEXT severity,
      VkDebugUtilsMessageTypeFlagsEXT type,
      const VkDebugUtilsMessengerCallbackDataEXT* data, void* user_data);

  VkInstance instance_;
  VkDebugUtilsMessengerEXT debug_messenger_;
};
