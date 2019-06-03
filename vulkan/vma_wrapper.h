#pragma once

#include "vulkan/vulkan_memory_allocator.h"

// https://gpuopen-librariesandsdks.github.io/VulkanMemoryAllocator/html/usage_patterns.html

class VMAWrapper {
 public:
  VMAWrapper(VkPhysicalDevice physical_device, VkDevice device);
  ~VMAWrapper();

  struct Allocation {
    VkBuffer buffer;
    VmaAllocation allocation;
  };

  // Staging buffer. Source from CPU. If source_data is not nullptr, then the
  // allocation will be mapped and the source data will be copied in.
  Allocation AllocateStagingBuffer(uint64_t size,
                                   const void* source_data = nullptr);

  void Free(Allocation all);

 private:
  static VmaAllocator ConstructAllocator(VkPhysicalDevice physical_device,
                                         VkDevice device);
  VmaAllocator allocator_;
};
