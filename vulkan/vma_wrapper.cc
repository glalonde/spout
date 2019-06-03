#include "vulkan/vma_wrapper.h"
#include <cstring>
#include "base/logging.h"
VMAWrapper::VMAWrapper(VkPhysicalDevice physical_device, VkDevice device)
    : allocator_(ConstructAllocator(physical_device, device)) {}

VMAWrapper::~VMAWrapper() {
  vmaDestroyAllocator(allocator_);
}

VmaAllocator VMAWrapper::ConstructAllocator(VkPhysicalDevice physical_device,
                                            VkDevice device) {
  VmaAllocatorCreateInfo allocator_info = {};
  allocator_info.physicalDevice = physical_device;
  allocator_info.device = device;
  VmaAllocator allocator;
  vmaCreateAllocator(&allocator_info, &allocator);
  return allocator;
}

VMAWrapper::Allocation VMAWrapper::AllocateStagingBuffer(
    uint64_t size, const void* source_data) {
  VkBufferCreateInfo buffer_info = {VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO};
  buffer_info.size = size;
  buffer_info.usage = VK_BUFFER_USAGE_TRANSFER_SRC_BIT;
  VmaAllocationCreateInfo alloc_info = {};
  alloc_info.usage = VMA_MEMORY_USAGE_CPU_ONLY;
  Allocation allocation;
  vmaCreateBuffer(allocator_, &buffer_info, &alloc_info, &allocation.buffer,
                  &allocation.allocation, nullptr);

  if (source_data != nullptr) {
    void* mapped_data;
    vmaMapMemory(allocator_, allocation.allocation, &mapped_data);
    std::memcpy(mapped_data, source_data, static_cast<size_t>(size));
    vmaUnmapMemory(allocator_, allocation.allocation);
  }

  return allocation;
}

void VMAWrapper::Free(Allocation all) {
  vmaDestroyBuffer(allocator_, all.buffer, all.allocation);
}
