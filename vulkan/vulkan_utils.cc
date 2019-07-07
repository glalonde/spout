#include "vulkan/vulkan_utils.h"
#include "base/file.h"
#include "base/logging.h"

VulkanDebugMessenger::VulkanDebugMessenger(VkInstance instance)
    : instance_(instance) {
  VkDebugUtilsMessengerCreateInfoEXT create_info = {};
  create_info.sType = VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT;
  create_info.messageSeverity =
      VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT |
      VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT |
      VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT;
  create_info.messageType = VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT |
                            VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT |
                            VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT;
  create_info.pfnUserCallback = DebugCallback;

  auto func = (PFN_vkCreateDebugUtilsMessengerEXT)vkGetInstanceProcAddr(
      instance_, "vkCreateDebugUtilsMessengerEXT");
  if (func != nullptr) {
    if (func(instance_, &create_info, nullptr, &debug_messenger_) !=
        VK_SUCCESS) {
      LOG(FATAL) << "Failed to setup debug messenger.";
    }
  } else {
    LOG(FATAL) << "Need to add extension: VK_EXT_DEBUG_UTILS_EXTENSION_NAME";
  }
}

VulkanDebugMessenger::~VulkanDebugMessenger() {
  auto func = (PFN_vkDestroyDebugUtilsMessengerEXT)vkGetInstanceProcAddr(
      instance_, "vkDestroyDebugUtilsMessengerEXT");
  if (func != nullptr) {
    func(instance_, debug_messenger_, nullptr);
  }
}

VKAPI_ATTR VkBool32 VKAPI_CALL VulkanDebugMessenger::DebugCallback(
    VkDebugUtilsMessageSeverityFlagBitsEXT severity,
    VkDebugUtilsMessageTypeFlagsEXT type,
    const VkDebugUtilsMessengerCallbackDataEXT* data, void* user_data) {
  switch (severity) {
    case VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT: {
      LOG(INFO) << data->pMessage;
      break;
    }
    case VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT: {
      LOG(INFO) << data->pMessage;
      break;
    }
    case VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT: {
      LOG(WARNING) << data->pMessage;
      break;
    }
    case VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT: {
      LOG(ERROR) << data->pMessage;
      break;
    }
    default:
      LOG(ERROR) << data->pMessage;
      break;
  }
  return VK_FALSE;
}

QueueFamilyIndices FindQueueFamilies(VkSurfaceKHR surface,
                                     VkPhysicalDevice device) {
  QueueFamilyIndices indices;

  uint32_t queue_family_count = 0;
  vkGetPhysicalDeviceQueueFamilyProperties(device, &queue_family_count,
                                           nullptr);

  std::vector<VkQueueFamilyProperties> queue_families(queue_family_count);
  vkGetPhysicalDeviceQueueFamilyProperties(device, &queue_family_count,
                                           queue_families.data());

  int i = 0;
  for (const auto& queue_family : queue_families) {
    // Graphics queue
    if (queue_family.queueCount > 0 &&
        queue_family.queueFlags & VK_QUEUE_GRAPHICS_BIT) {
      indices.graphics_family = i;
    }

    // Compute queue
    if (queue_family.queueCount > 0 &&
        queue_family.queueFlags & VK_QUEUE_COMPUTE_BIT) {
      indices.compute_family = i;
    }

    // Present queue
    if (surface != VK_NULL_HANDLE) {
      VkBool32 present_support = false;
      vkGetPhysicalDeviceSurfaceSupportKHR(device, i, surface,
                                           &present_support);
      if (queue_family.queueCount > 0 && present_support) {
        indices.present_family = i;
      }
    }

    i++;
  }

  return indices;
}

SwapChainSupportDetails QuerySwapChainSupport(VkSurfaceKHR surface,
                                              VkPhysicalDevice device) {
  SwapChainSupportDetails details;

  vkGetPhysicalDeviceSurfaceCapabilitiesKHR(device, surface,
                                            &details.capabilities);

  uint32_t format_count;
  vkGetPhysicalDeviceSurfaceFormatsKHR(device, surface, &format_count, nullptr);

  if (format_count != 0) {
    details.formats.resize(format_count);
    vkGetPhysicalDeviceSurfaceFormatsKHR(device, surface, &format_count,
                                         details.formats.data());
  }

  uint32_t present_mode_count;
  vkGetPhysicalDeviceSurfacePresentModesKHR(device, surface,
                                            &present_mode_count, nullptr);

  if (present_mode_count != 0) {
    details.present_modes.resize(present_mode_count);
    vkGetPhysicalDeviceSurfacePresentModesKHR(
        device, surface, &present_mode_count, details.present_modes.data());
  }

  return details;
}

ErrorXor<VkShaderModule> CreateShaderModule(VkDevice device,
                                            const std::string_view& path) {
  auto maybe_shader_code = TryReadFile(path);
  if (!maybe_shader_code) {
    return std::move(*maybe_shader_code.ErrorOrNull());
  }
  const std::string& code = maybe_shader_code.ValueOrDie();
  VkShaderModuleCreateInfo create_info = {};
  create_info.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
  create_info.codeSize = code.size();
  create_info.pCode = reinterpret_cast<const uint32_t*>(&code);

  VkShaderModule shader_module;
  if (vkCreateShaderModule(device, &create_info, nullptr, &shader_module) !=
      VK_SUCCESS) {
    return TraceError("Failed to create shader module.");
  }
  return shader_module;
}
