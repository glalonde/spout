#include "vulkan/vulkan_utils.h"
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
