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

VMAWrapper::Buffer VMAWrapper::AllocateStagingBuffer(uint64_t size,
                                                     const void* source_data) {
  Buffer buffer = CreateBuffer(
      size, VK_BUFFER_USAGE_TRANSFER_SRC_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT,
      VMA_MEMORY_USAGE_CPU_ONLY, {});

  if (source_data != nullptr) {
    CopyToBuffer(buffer, source_data, size);
  }

  return buffer;
}

void VMAWrapper::Free(Buffer all) {
  vmaDestroyBuffer(allocator_, all.buffer, all.allocation);
}

VMAWrapper::Buffer VMAWrapper::CreateGPUBuffer(
    uint64_t size, VkBufferUsageFlags usage,
    std::vector<uint32_t> queue_families) {
  return CreateBuffer(size, usage, VMA_MEMORY_USAGE_GPU_ONLY, queue_families);
}

VMAWrapper::Buffer VMAWrapper::CreateCPUToGPUBuffer(
    uint64_t size, VkBufferUsageFlags usage,
    std::vector<uint32_t> queue_families) {
  return CreateBuffer(size, usage, VMA_MEMORY_USAGE_CPU_TO_GPU, queue_families);
}

VMAWrapper::Buffer VMAWrapper::CreateBuffer(
    uint64_t size, VkBufferUsageFlags usage, VmaMemoryUsage vma_usage,
    const std::vector<uint32_t>& queue_families) {
  VkBufferCreateInfo buffer_info = {VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO};
  buffer_info.size = size;
  buffer_info.usage = usage;
  if (queue_families.size() > 1) {
    buffer_info.sharingMode = VK_SHARING_MODE_CONCURRENT;
    buffer_info.queueFamilyIndexCount = queue_families.size();
    buffer_info.pQueueFamilyIndices = queue_families.data();
  } else {
    buffer_info.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
  }
  VmaAllocationCreateInfo alloc_info = {};
  alloc_info.usage = vma_usage;
  Buffer buffer;
  vmaCreateBuffer(allocator_, &buffer_info, &alloc_info, &buffer.buffer,
                  &buffer.allocation, nullptr);
  return buffer;
}

void VMAWrapper::CopyToBuffer(Buffer buffer, const void* source_data,
                              size_t size) {
  void* mapped_data;
  vmaMapMemory(allocator_, buffer.allocation, &mapped_data);
  std::memcpy(mapped_data, source_data, size);
  vmaUnmapMemory(allocator_, buffer.allocation);
}

void VMAWrapper::MapBuffer(Buffer buffer, void** mapped_data) {
  vmaMapMemory(allocator_, buffer.allocation, mapped_data);
}

void VMAWrapper::UnmapBuffer(Buffer buffer) {
  vmaUnmapMemory(allocator_, buffer.allocation);
}

void PopulateVkImageStructures(
    const absl::InlinedVector<uint32_t, 3>& dimensions, VkImageType* type,
    VkExtent3D* extents) {
  static constexpr std::array<VkImageType, 3> kSizeToType = {
      VK_IMAGE_TYPE_1D, VK_IMAGE_TYPE_2D, VK_IMAGE_TYPE_3D};
  *type = kSizeToType[dimensions.size()];
  absl::InlinedVector<uint32_t, 3> dimensions_padded = dimensions;
  while (dimensions_padded.size() < 3) {
    dimensions_padded.push_back(-1);
  }
  extents->width = dimensions_padded[0];
  extents->height = dimensions_padded[1];
  extents->depth = dimensions_padded[2];
}

VMAWrapper::Image VMAWrapper::CreateImage(
    const absl::InlinedVector<uint32_t, 3>& dimensions, VkFormat format,
    VkImageUsageFlags usage, VmaMemoryUsage vma_usage,
    const std::vector<uint32_t>& queue_families) {
  VkImageCreateInfo image_info = {VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO};
  PopulateVkImageStructures(dimensions, &image_info.imageType,
                            &image_info.extent);
  image_info.usage = usage;
  if (queue_families.size() > 1) {
    image_info.sharingMode = VK_SHARING_MODE_CONCURRENT;
    image_info.queueFamilyIndexCount = queue_families.size();
    image_info.pQueueFamilyIndices = queue_families.data();
  } else {
    image_info.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
  }
  VmaAllocationCreateInfo alloc_info = {};
  alloc_info.usage = vma_usage;
  Image image;
  vmaCreateImage(allocator_, &image_info, &alloc_info, &image.image,
                 &image.allocation, nullptr);
  return image;
}
