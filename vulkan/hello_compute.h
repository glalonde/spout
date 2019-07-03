#pragma once
#include <vulkan/vulkan.h>
#include <vector>

#include "vulkan/vma_wrapper.h"
#include "vulkan/vulkan_utils.h"

class ComputeApplication {
 public:
  ComputeApplication();
  void Run(int width, int height);

 private:
  std::vector<const char*> GetRequiredInstanceExtensions();
  void InitVulkan();
  void CreateInstance();
  void MaybeInitDebugMessenger();
  void PickPhysicalDevice();
  void CreateLogicalDevice();
  void CreateAllocator();
  void MakeBuffers();

  VkInstance instance_;
  std::unique_ptr<VulkanDebugMessenger> debug_messenger_;
  VkPhysicalDevice physical_device_;
  VkDevice device_;
  VkQueue compute_queue_;
  std::unique_ptr<VMAWrapper> allocator_;
  int width_;
  int height_;
  VMAWrapper::Buffer storage_buffer_;
  VMAWrapper::Buffer staging_buffer_;
};
