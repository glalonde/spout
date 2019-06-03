#pragma once

#include "vulkan/vulkan_memory_allocator.h"

// https://gpuopen-librariesandsdks.github.io/VulkanMemoryAllocator/html/usage_patterns.html

class VMAWrapper {
 public:
  VMAWrapper(VkPhysicalDevice physical_device, VkDevice device);
  ~VMAWrapper();

  struct Buffer {
    VkBuffer buffer;
    VmaAllocation allocation;
  };

  // Staging buffer. Source from CPU. If source_data is not nullptr, then the
  // allocation will be mapped and the source data will be copied in.
  Buffer AllocateStagingBuffer(uint64_t size,
                               const void* source_data = nullptr);

  Buffer CreateGPUBuffer(uint64_t size, VkBufferUsageFlags usage);

  // For direct CPU to GPU mapping (no staging / explicit transfer)
  Buffer CreateCPUToGPUBuffer(uint64_t size, VkBufferUsageFlags usage);

  void CopyToBuffer(Buffer buffer, const void* source_data, size_t size);

  void Free(Buffer all);

 private:
  static VmaAllocator ConstructAllocator(VkPhysicalDevice physical_device,
                                         VkDevice device);

  Buffer CreateBuffer(uint64_t size, VkBufferUsageFlags usage,
                      VmaMemoryUsage vma_usage);

  VmaAllocator allocator_;
};
