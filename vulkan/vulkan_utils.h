#pragma once
#include <vulkan/vulkan.h>
#include <optional>
#include <vector>
#include "absl/algorithm/container.h"
#include "base/logging.h"

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

// No idea what this is.
struct QueueFamilyIndices {
  std::optional<uint32_t> graphics_family;
  std::optional<uint32_t> present_family;

  bool is_complete() {
    return graphics_family.has_value() && present_family.has_value();
  }
};
QueueFamilyIndices FindQueueFamilies(VkSurfaceKHR surface,
                                     VkPhysicalDevice device);

// No idea what this is.
struct SwapChainSupportDetails {
  VkSurfaceCapabilitiesKHR capabilities;
  std::vector<VkSurfaceFormatKHR> formats;
  std::vector<VkPresentModeKHR> present_modes;
};
SwapChainSupportDetails QuerySwapChainSupport(VkSurfaceKHR surface,
                                              VkPhysicalDevice device);

bool IsDeviceSuitable(
    VkSurfaceKHR surface, VkPhysicalDevice device,
    const std::vector<const char*>& required_device_extensions);

// Returns whether the device supports all the required extensions.
template <typename C>
bool CheckDeviceExtensionSupport(VkPhysicalDevice device,
                                 C&& required_extensions) {
  uint32_t extension_count;
  vkEnumerateDeviceExtensionProperties(device, nullptr, &extension_count,
                                       nullptr);
  std::vector<VkExtensionProperties> available_extensions(extension_count);
  vkEnumerateDeviceExtensionProperties(device, nullptr, &extension_count,
                                       available_extensions.data());
  auto extension_found = [&available_extensions](const auto& req) {
    return absl::c_find_if(available_extensions, [&req](const auto& ext) {
             return req == std::string_view(ext.extensionName);
           }) != available_extensions.end();
  };
  return absl::c_all_of(required_extensions, extension_found);
}
