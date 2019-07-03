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
