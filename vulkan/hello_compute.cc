#include "vulkan/hello_compute.h"
#include "src/convert.h"
#include "src/image.h"
#include "src/image_io.h"

#ifdef NDEBUG
static constexpr bool kVulkanDebugMode = false;
#else
static constexpr bool kVulkanDebugMode = true;
#endif

static const std::vector<const char*> kValidationLayers = {
    "VK_LAYER_KHRONOS_validation"};

static const std::vector<const char*> kDeviceExtensions = {};

ComputeApplication::ComputeApplication() {}

void ComputeApplication::Run(int width, int height,
                             const std::string& dest_path) {
  LOG(INFO) << "Running with width=" << width << ", height=" << height;
  width_ = width;
  height_ = height;
  workgroup_size_ = 32;
  InitVulkan();
  MakeBuffers();
  CreateDescriptorSetLayout();
  CreateDescriptorPool();
  CreateDescriptorSet();
  CreateComputePipeline();
  CreateCommandPool();
  CreateCommandBuffer();
  RunCommandBuffer();
  SaveRenderedImage(dest_path);
  Cleanup();
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
      buffer_size,
      VK_BUFFER_USAGE_STORAGE_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_SRC_BIT);
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
  buffer_info.range = VK_WHOLE_SIZE;

  VkWriteDescriptorSet descriptor_write = {};
  descriptor_write.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
  descriptor_write.dstSet = descriptor_set_;
  descriptor_write.dstBinding = 0;
  descriptor_write.descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
  descriptor_write.descriptorCount = 1;
  descriptor_write.pBufferInfo = &buffer_info;
  vkUpdateDescriptorSets(device_, 1, &descriptor_write, 0, nullptr);
}

void ComputeApplication::CreateComputePipeline() {
  // Compile shaders
  VkShaderModule shader_module =
      CreateShaderModule(device_, "vulkan/shaders/mandelbrot.comp.spv")
          .ValueOrDie();

  VkPipelineShaderStageCreateInfo shader_stage_info = {};
  shader_stage_info.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
  shader_stage_info.stage = VK_SHADER_STAGE_COMPUTE_BIT;
  shader_stage_info.module = shader_module;
  shader_stage_info.pName = "main";

  // The pipeline layout allows the pipeline to access descriptor sets. So we
  // just specify the descriptor set layout we created earlier.
  VkPipelineLayoutCreateInfo pipeline_layout_info = {};
  pipeline_layout_info.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
  pipeline_layout_info.setLayoutCount = 1;
  pipeline_layout_info.pSetLayouts = &descriptor_set_layout_;
  if (vkCreatePipelineLayout(device_, &pipeline_layout_info, nullptr,
                             &pipeline_layout_) != VK_SUCCESS) {
    LOG(FATAL) << "Failed to create pipeline layout.";
  }

  VkComputePipelineCreateInfo pipeline_info = {};
  pipeline_info.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
  pipeline_info.stage = shader_stage_info;
  pipeline_info.layout = pipeline_layout_;

  if (vkCreateComputePipelines(device_, VK_NULL_HANDLE, 1, &pipeline_info,
                               nullptr, &compute_pipeline_) != VK_SUCCESS) {
    LOG(FATAL) << "Failed to create compute pipeline.";
  }
  vkDestroyShaderModule(device_, shader_module, nullptr);
}

void ComputeApplication::CreateCommandPool() {
  QueueFamilyIndices queue_family_indices =
      FindQueueFamilies(nullptr, physical_device_);
  VkCommandPoolCreateInfo pool_info = {};
  pool_info.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
  pool_info.queueFamilyIndex = queue_family_indices.compute_family.value();
  if (vkCreateCommandPool(device_, &pool_info, nullptr, &command_pool_) !=
      VK_SUCCESS) {
    LOG(FATAL) << "Failed to create command pool.";
  }
}

void ComputeApplication::CreateCommandBuffer() {
  VkCommandBufferAllocateInfo alloc_info = {};
  alloc_info.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
  alloc_info.commandPool = command_pool_;
  // if the command buffer is primary, it can be directly submitted to queues.
  // A secondary buffer has to be called from some primary command buffer, and
  // cannot be directly submitted to a queue. To keep things simple, we use a
  // primary command buffer.
  alloc_info.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
  alloc_info.commandBufferCount = 1;
  if (vkAllocateCommandBuffers(device_, &alloc_info, &command_buffer_) !=
      VK_SUCCESS) {
    LOG(FATAL) << "Failed to allocate command buffer.";
  }

  // Start recording commands into the newly allocated command buffer.
  VkCommandBufferBeginInfo begin_info = {};
  begin_info.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
  // Only submitted and used once.
  begin_info.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
  if (vkBeginCommandBuffer(command_buffer_, &begin_info) != VK_SUCCESS) {
    LOG(FATAL) << "Failed to begin recording rommand buffer.";
  }

  // We need to bind a pipeline, AND a descriptor set before we dispatch. The
  // validation layer will NOT give warnings if you forget these, so be very
  // careful not to forget them.
  vkCmdBindPipeline(command_buffer_, VK_PIPELINE_BIND_POINT_COMPUTE,
                    compute_pipeline_);
  vkCmdBindDescriptorSets(command_buffer_, VK_PIPELINE_BIND_POINT_COMPUTE,
                          pipeline_layout_, 0, 1, &descriptor_set_, 0, nullptr);

  // Calling vkCmdDispatch basically starts the compute pipeline, and executes
  // the compute shader.
  vkCmdDispatch(
      command_buffer_,
      static_cast<uint32_t>(std::ceil(width_ / float(workgroup_size_))),
      static_cast<uint32_t>(std::ceil(height_ / float(workgroup_size_))), 1);

  if (vkEndCommandBuffer(command_buffer_) != VK_SUCCESS) {
    LOG(FATAL) << "Failed to record command buffer.";
  }
}

void ComputeApplication::RunCommandBuffer() {
  VkSubmitInfo submit_info = {};
  submit_info.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
  submit_info.commandBufferCount = 1;
  submit_info.pCommandBuffers = &command_buffer_;

  VkFence fence;
  VkFenceCreateInfo fence_create_info = {};
  fence_create_info.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
  fence_create_info.flags = 0;
  if (vkCreateFence(device_, &fence_create_info, NULL, &fence) != VK_SUCCESS) {
    LOG(FATAL) << "Failed to create fence.";
  }

  if (vkQueueSubmit(compute_queue_, 1, &submit_info, fence) != VK_SUCCESS) {
    LOG(FATAL) << "Failed to submit compute command buffer.";
  }
  // The command will not have finished executing until the fence is signalled.
  if (vkWaitForFences(device_, 1, &fence, VK_TRUE, 100000000000) !=
      VK_SUCCESS) {
    LOG(FATAL) << "Failed to wait for fence.";
  }
  vkDestroyFence(device_, fence, NULL);
}

void ComputeApplication::CopyBuffer(VkBuffer src_buff, VkBuffer dest_buff,
                                    VkDeviceSize size) {
  VkCommandBufferAllocateInfo alloc_info = {};
  alloc_info.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
  alloc_info.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
  alloc_info.commandPool = command_pool_;
  alloc_info.commandBufferCount = 1;

  VkCommandBuffer command_buffer;
  vkAllocateCommandBuffers(device_, &alloc_info, &command_buffer);

  VkCommandBufferBeginInfo begin_info = {};
  begin_info.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
  begin_info.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
  vkBeginCommandBuffer(command_buffer, &begin_info);

  VkBufferCopy copy_region = {};
  copy_region.srcOffset = 0;  // Optional
  copy_region.dstOffset = 0;  // Optional
  copy_region.size = size;
  vkCmdCopyBuffer(command_buffer, src_buff, dest_buff, 1, &copy_region);
  vkEndCommandBuffer(command_buffer);

  VkSubmitInfo submit_info = {};
  submit_info.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
  submit_info.commandBufferCount = 1;
  submit_info.pCommandBuffers = &command_buffer;

  vkQueueSubmit(compute_queue_, 1, &submit_info, VK_NULL_HANDLE);
  vkQueueWaitIdle(compute_queue_);
  vkFreeCommandBuffers(device_, command_pool_, 1, &command_buffer);
}

void ComputeApplication::SaveRenderedImage(const std::string& dest_path) {
  // Copy from storage to staging
  const int size = storage_buffer_.GetSize(*allocator_);
  CopyBuffer(storage_buffer_.buffer, staging_buffer_.buffer, size);
  constexpr int kNumChannels = 4;
  Image<PixelType::RGBAU8> out(height_, width_);
  {
    // Map staging to an local memory
    void* mapped_data;
    allocator_->MapBuffer(staging_buffer_, &mapped_data);

    // Map the local memory to an eigen matrix.
    Eigen::Map<
        Eigen::Array<float, Eigen::Dynamic, Eigen::Dynamic, Eigen::RowMajor>>
    mapped_array(static_cast<float*>(mapped_data), height_,
                 width_ * kNumChannels);
    for (int i = 0; i < height_; ++i) {
      for (int j = 0; j < width_; ++j) {
        out(i, j) = Convert<PixelType::RGBAU8, PixelType::RGBAF32>(
            mapped_array.block<1, kNumChannels>(i, j * kNumChannels));
      }
    }
    allocator_->UnmapBuffer(staging_buffer_);
  }
  if (WriteImage(out, dest_path)) {
    LOG(INFO) << "Successfully wrote image to: " << dest_path;
  } else {
    LOG(ERROR) << "Failed to write image to: " << dest_path;
  }
}

void ComputeApplication::Cleanup() {
  vkFreeCommandBuffers(device_, command_pool_, 1, &command_buffer_);
  vkDestroyPipeline(device_, compute_pipeline_, nullptr);
  vkDestroyPipelineLayout(device_, pipeline_layout_, nullptr);
  vkDestroyDescriptorPool(device_, descriptor_pool_, nullptr);
  vkDestroyDescriptorSetLayout(device_, descriptor_set_layout_, nullptr);
  allocator_->Free(storage_buffer_);
  allocator_->Free(staging_buffer_);
  allocator_.reset();

  vkDestroyCommandPool(device_, command_pool_, nullptr);
  vkDestroyDevice(device_, nullptr);
  debug_messenger_.reset();
  vkDestroyInstance(instance_, nullptr);
}
