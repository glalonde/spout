#pragma once
#include "vulkan/glfw.h"

#include <algorithm>
#include <array>
#include <cstdlib>
#include <cstring>
#include <optional>
#include <vector>

#include "src/eigen_types.h"
#include "src/fps_estimator.h"
#include "vulkan/vma_wrapper.h"
#include "vulkan/vulkan_utils.h"

struct Vertex {
  Vector2f position;
  Vector3f color;

  static VkVertexInputBindingDescription GetBindingDescription() {
    VkVertexInputBindingDescription desc = {};
    desc.binding = 0;
    desc.stride = sizeof(Vertex);
    desc.inputRate = VK_VERTEX_INPUT_RATE_VERTEX;
    return desc;
  }

  static std::array<VkVertexInputAttributeDescription, 2>
  GetAttributeDescriptions() {
    std::array<VkVertexInputAttributeDescription, 2> desc = {};
    desc[0].binding = 0;
    desc[0].location = 0;
    desc[0].format = VK_FORMAT_R32G32_SFLOAT;
    desc[0].offset = offsetof(Vertex, position);

    desc[1].binding = 0;
    desc[1].location = 1;
    desc[1].format = VK_FORMAT_R32G32B32_SFLOAT;
    desc[1].offset = offsetof(Vertex, color);

    return desc;
  }
};

struct UniformBufferObject {
  Matrix4f model;
  Matrix4f view;
  Matrix4f proj;
};

const std::vector<Vertex> kVertices = {{{-0.5f, -0.5f}, {1.0f, 0.0f, 0.0f}},
                                       {{0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
                                       {{0.5f, 0.5f}, {0.0f, 0.0f, 1.0f}},
                                       {{-0.5f, 0.5f}, {1.0f, 1.0f, 1.0f}}};

const std::vector<uint16_t> kIndices = {0, 1, 2, 2, 3, 0};

class ImageViewerApplication {
 public:
  ImageViewerApplication();
  void Run();

 private:
  GLFWwindow* window_;
  FPSEstimator fps_;

  VkInstance instance_;
  std::unique_ptr<VulkanDebugMessenger> debug_messenger_;
  VkSurfaceKHR surface_;

  VkPhysicalDevice physical_device_ = VK_NULL_HANDLE;
  VkDevice device_;

  VkQueue graphics_queue_;
  VkQueue present_queue_;

  VkSwapchainKHR swap_chain_;
  std::vector<VkImage> swap_chain_images_;
  VkFormat swap_chain_image_format_;
  VkExtent2D swap_chain_extent_;
  std::vector<VkImageView> swap_chain_image_views_;
  std::vector<VkFramebuffer> swap_chain_frame_buffers_;

  VkRenderPass render_pass_;
  VkDescriptorSetLayout descriptor_set_layout_;
  VkPipelineLayout pipeline_layout_;
  VkPipeline graphics_pipeline_;

  VkCommandPool command_pool_;
  VkDescriptorPool descriptor_pool_;
  std::vector<VkDescriptorSet> descriptor_sets_;

  std::unique_ptr<VMAWrapper> allocator_;
  VMAWrapper::Buffer vertex_buffer_;
  VMAWrapper::Buffer index_buffer_;
  std::vector<VMAWrapper::Buffer> uniform_buffers_;
  VkImage texture_image_;
  VMAWrapper::Buffer texture_image_memory_;

  std::vector<VkCommandBuffer> command_buffers_;

  std::vector<VkSemaphore> image_available_semaphores_;
  std::vector<VkSemaphore> render_finished_semaphores_;
  std::vector<VkFence> in_flight_fences_;
  size_t current_frame_ = 0;

  bool framebuffer_resized_ = false;

  void InitWindow();
  static void FramebufferResizeCallback(GLFWwindow* window, int width,
                                        int height);
  void InitVulkan();
  void CreateAllocator();
  void MainLoop();
  void CleanupSwapChain();
  void Cleanup();
  void RecreateSwapChain();
  void CreateInstance();
  void CreateSurface();
  void PickPhysicalDevice();
  void CreateLogicalDevice();
  void CreateSwapChain();
  void CreateImageViews();
  void CreateRenderPass();
  void CreateDescriptorSetLayout();
  void CreateGraphicsPipeline();
  void CreateFramebuffers();
  void CreateCommandPool();
  void CreateDescriptorPool();
  void CreateDescriptorSets();
  void CreateVertexBuffer();
  void CreateIndexBuffer();
  void CreateUniformBuffers();
  void CopyBuffer(VkBuffer src_buff, VkBuffer dest_buff, VkDeviceSize size);
  void CreateCommandBuffers();
  void CreateSyncObjects();
  void DrawFrame();
  void UpdateUniformBuffer(uint32_t current_image);
  VkSurfaceFormatKHR ChooseSwapSurfaceFormat(
      const std::vector<VkSurfaceFormatKHR>& available_formats);
  VkPresentModeKHR ChooseSwapPresentMode(
      const std::vector<VkPresentModeKHR>& available_present_modes);
  VkExtent2D ChooseSwapExtent(const VkSurfaceCapabilitiesKHR& capabilities);
  std::vector<const char*> GetRequiredExtensions();
};
