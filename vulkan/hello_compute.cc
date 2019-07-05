#include "vulkan/hello_compute.h"
#include "src/image.h"

#ifdef NDEBUG
static constexpr bool kVulkanDebugMode = false;
#else
static constexpr bool kVulkanDebugMode = true;
#endif

static const std::vector<const char*> kValidationLayers = {
    "VK_LAYER_KHRONOS_validation"};

static const std::vector<const char*> kDeviceExtensions = {};

ComputeApplication::ComputeApplication() {}

void ComputeApplication::Run(int width, int height) {
  LOG(INFO) << "Running with width=" << width << ", height=" << height;
  width_ = width;
  height_ = height;
  InitVulkan();
  MakeBuffers();
  CreateDescriptorSetLayout();
}

void ComputeApplication::InitVulkan() {
  CreateInstance();
  MaybeInitDebugMessenger();
  PickPhysicalDevice();
  CreateLogicalDevice();
  CreateAllocator();
}

std::vector<const char*> ComputeApplication::GetRequiredInstanceExtensions() {
  std::vector<const char*> extensions;
  if (kVulkanDebugMode) {
    extensions.push_back(VK_EXT_DEBUG_UTILS_EXTENSION_NAME);
  }
  return extensions;
}

void ComputeApplication::CreateInstance() {
  if (kVulkanDebugMode && !CheckValidationLayerSupport(kValidationLayers)) {
    LOG(FATAL) << "Validation layers requested but not available.";
  }

  VkApplicationInfo app_info = {};
  app_info.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
  app_info.pApplicationName = "Hello Compute";
  app_info.applicationVersion = VK_MAKE_VERSION(1, 0, 0);
  app_info.pEngineName = "No Engine";
  app_info.engineVersion = VK_MAKE_VERSION(1, 0, 0);
  app_info.apiVersion = VK_API_VERSION_1_0;

  VkInstanceCreateInfo create_info = {};
  create_info.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
  create_info.pApplicationInfo = &app_info;

  const auto extensions = GetRequiredInstanceExtensions();
  create_info.enabledExtensionCount = static_cast<uint32_t>(extensions.size());
  create_info.ppEnabledExtensionNames = extensions.data();
  create_info.enabledLayerCount =
      static_cast<uint32_t>(kValidationLayers.size());
  create_info.ppEnabledLayerNames = kValidationLayers.data();

  if (vkCreateInstance(&create_info, nullptr, &instance_) != VK_SUCCESS) {
    LOG(FATAL) << "Failed to create instance.";
  }
  LOG(INFO) << "Created instance.";
}

void ComputeApplication::MaybeInitDebugMessenger() {
  if (kVulkanDebugMode) {
    LOG(INFO) << "Running in DEBUG mode.";
    debug_messenger_ = std::make_unique<VulkanDebugMessenger>(instance_);
  }
}

void ComputeApplication::PickPhysicalDevice() {
  physical_device_ = FindPhysicalDevice(instance_, [](VkPhysicalDevice device) {
    QueueFamilyIndices indices = FindQueueFamilies(VK_NULL_HANDLE, device);
    if (!indices.compute_family) {
      return false;
    }
    return CheckDeviceExtensionSupport(device, kDeviceExtensions);
  });
}

void ComputeApplication::CreateLogicalDevice() {
  QueueFamilyIndices indices =
      FindQueueFamilies(VK_NULL_HANDLE, physical_device_);
  CHECK(indices.compute_family);
  const float queue_priority = 1.0f;
  VkDeviceQueueCreateInfo queue_create_info = {};
  queue_create_info.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
  queue_create_info.queueFamilyIndex = *indices.compute_family;
  queue_create_info.queueCount = 1;
  queue_create_info.pQueuePriorities = &queue_priority;

  VkPhysicalDeviceFeatures device_features = {};

  VkDeviceCreateInfo create_info = {};
  create_info.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
  create_info.queueCreateInfoCount = 1;
  create_info.pQueueCreateInfos = &queue_create_info;
  create_info.pEnabledFeatures = &device_features;
  create_info.enabledExtensionCount =
      static_cast<uint32_t>(kDeviceExtensions.size());
  create_info.ppEnabledExtensionNames = kDeviceExtensions.data();
  create_info.enabledLayerCount =
      static_cast<uint32_t>(kValidationLayers.size());
  create_info.ppEnabledLayerNames = kValidationLayers.data();

  if (vkCreateDevice(physical_device_, &create_info, nullptr, &device_) !=
      VK_SUCCESS) {
    LOG(FATAL) << "Failed to create logical device.";
  }

  vkGetDeviceQueue(device_, *indices.compute_family, 0, &compute_queue_);
}

void ComputeApplication::CreateAllocator() {
  allocator_ = std::make_unique<VMAWrapper>(physical_device_, device_);
}

void ComputeApplication::MakeBuffers() {
  VkDeviceSize buffer_size = sizeof(PixelType::RGBAF32) * width_ * height_;
  storage_buffer_ = allocator_->CreateGPUBuffer(
      buffer_size, VK_BUFFER_USAGE_STORAGE_BUFFER_BIT);
  staging_buffer_ = allocator_->AllocateStagingBuffer(buffer_size, nullptr);
}

void ComputeApplication::CreateDescriptorSetLayout() {
  // The descriptor set layout allows us to specify how to access the buffer
  // from the shader. This binds to:
  // layout(std140, binding = 0) buffer buf
  VkDescriptorSetLayoutBinding layout_binding = {};
  layout_binding.binding = 0;
  layout_binding.descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
  layout_binding.descriptorCount = 1;
  layout_binding.stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;

  VkDescriptorSetLayoutCreateInfo layout_info = {};
  layout_info.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
  layout_info.bindingCount = 1;
  layout_info.pBindings = &layout_binding;

  if (vkCreateDescriptorSetLayout(device_, &layout_info, nullptr,
                                  &descriptor_set_layout_) != VK_SUCCESS) {
    LOG(FATAL) << "Failed to create descriptor set layout.";
  }
}

void ComputeApplication::CreateDescriptorPool() {
  VkDescriptorPoolSize pool_size = {};
  pool_size.type = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
  pool_size.descriptorCount = 1;
  VkDescriptorPoolCreateInfo pool_info = {};
  pool_info.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO;
  pool_info.poolSizeCount = 1;
  pool_info.pPoolSizes = &pool_size;
  pool_info.maxSets = 1;

  if (vkCreateDescriptorPool(device_, &pool_info, nullptr, &descriptor_pool_) !=
      VK_SUCCESS) {
    LOG(FATAL) << "Failed to create descriptor pool.";
  }
}

void ComputeApplication::CreateDescriptorSet() {
  // Allocate the descriptor set from the pool
  VkDescriptorSetAllocateInfo alloc_info = {};
  alloc_info.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO;
  alloc_info.descriptorPool = descriptor_pool_;
  alloc_info.descriptorSetCount = 1;
  alloc_info.pSetLayouts = &descriptor_set_layout_;
  if (vkAllocateDescriptorSets(device_, &alloc_info, &descriptor_set_) !=
      VK_SUCCESS) {
    LOG(FATAL) << "Failed to allocate descriptor sets.";
  }

  // Connect the buffer to the descriptor set
  VkDescriptorBufferInfo buffer_info = {};
  buffer_info.buffer = storage_buffer_.buffer;
  buffer_info.offset = 0;
  buffer_info.range = sizeof(UniformBufferObject);

  VkWriteDescriptorSet descriptor_write = {};
  descriptor_write.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
  descriptor_write.dstSet = descriptor_sets_[i];
  descriptor_write.dstBinding = 0;
  descriptor_write.dstArrayElement = 0;
  descriptor_write.descriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER;
  descriptor_write.descriptorCount = 1;
  descriptor_write.pBufferInfo = &buffer_info;
  descriptor_write.pImageInfo = nullptr;        // Optional
  descriptor_write.pTexelBufferView = nullptr;  // Optional
  vkUpdateDescriptorSets(device_, 1, &descriptor_write, 0, nullptr);
}
