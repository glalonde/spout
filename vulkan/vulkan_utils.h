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

// Get handles to families that support various operations
struct QueueFamilyIndices {
  std::optional<uint32_t> graphics_family;
  std::optional<uint32_t> present_family;
  std::optional<uint32_t> compute_family;
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

// Returns whether the environment supports all the required validation layers.
template <typename C>
bool CheckValidationLayerSupport(C&& required_layers) {
  uint32_t layer_count;
  vkEnumerateInstanceLayerProperties(&layer_count, nullptr);

  std::vector<VkLayerProperties> available_layers(layer_count);
  vkEnumerateInstanceLayerProperties(&layer_count, available_layers.data());
  auto layer_found = [&available_layers](const auto& req) {
    return absl::c_find_if(available_layers, [&req](const auto& layer) {
             return req == std::string_view(layer.layerName);
           }) != available_layers.end();
  };
  return absl::c_all_of(required_layers, layer_found);
}

template<class F>
VkPhysicalDevice FindPhysicalDevice(VkInstance instance, F filter) {
  uint32_t device_count = 0;
  vkEnumeratePhysicalDevices(instance, &device_count, nullptr);
  if (device_count == 0) {
    LOG(FATAL) << "Failed to find GPU with Vulkan support.";
  }
  std::vector<VkPhysicalDevice> devices(device_count);
  vkEnumeratePhysicalDevices(instance, &device_count, devices.data());
  for (const auto& device : devices) {
    if (filter(device)) {
      return device;
    }
  }
  LOG(FATAL) << "Failed to find suitable GPU.";
}
