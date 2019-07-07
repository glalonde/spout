#pragma once

#include <vector>

#include "vulkan/vulkan_memory_allocator.h"

// https://gpuopen-librariesandsdks.github.io/VulkanMemoryAllocator/html/usage_patterns.html

class VMAWrapper {
 public:
  VMAWrapper(VkPhysicalDevice physical_device, VkDevice device);
  ~VMAWrapper();

  struct Buffer {
    VkBuffer buffer;
    VmaAllocation allocation;

    // TODO(glalonde) consider stashing the pointer to the allocator internally,
    // and maybe caching stuff like size.
    VmaAllocationInfo GetInfo(const VMAWrapper& allocator) const {
      VmaAllocationInfo info;
      vmaGetAllocationInfo(allocator.allocator_, allocation,
                           &info);
      return info;
    }
    VkDeviceSize GetSize(const VMAWrapper& allocator) const {
      return GetInfo(allocator).size;
    }
  };

  // Staging buffer. Source from CPU. If source_data is not nullptr, then the
  // allocation will be mapped and the source data will be copied in.
  Buffer AllocateStagingBuffer(uint64_t size,
                               const void* source_data = nullptr);

  Buffer CreateGPUBuffer(uint64_t size, VkBufferUsageFlags usage,
                         std::vector<uint32_t> queue_families = {});

  // For direct CPU to GPU mapping (no staging / explicit transfer)
  Buffer CreateCPUToGPUBuffer(uint64_t size, VkBufferUsageFlags usage,
                              std::vector<uint32_t> queue_families = {});

  void CopyToBuffer(Buffer buffer, const void* source_data, size_t size);

  void Free(Buffer all);

 private:
  static VmaAllocator ConstructAllocator(VkPhysicalDevice physical_device,
                                         VkDevice device);

  // Pass a vector of queue_family indices if this is to be used by multiple
  // queue families (sets sharing mode to CONCURRENT, otherwise its EXCLUSIVE)
  Buffer CreateBuffer(uint64_t size, VkBufferUsageFlags usage,
                      VmaMemoryUsage vma_usage,
                      const std::vector<uint32_t>& queue_families);

  VmaAllocator allocator_;
};
