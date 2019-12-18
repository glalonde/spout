#pragma once
#define GLFW_INCLUDE_VULKAN
#include <GLFW/glfw3.h>
#include "src/eigen_types.h"
#include "src/fps_estimator.h"
#include "vulkan/vma_wrapper.h"
#include "vulkan/vulkan_utils.h"

class GameController {
 public:
  GameController();
  void Run();

 private:
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

  std::vector<VkCommandBuffer> command_buffers_;

  std::vector<VkSemaphore> image_available_semaphores_;
  std::vector<VkSemaphore> render_finished_semaphores_;
  std::vector<VkFence> in_flight_fences_;
  size_t current_frame_;
  bool framebuffer_resized_;
};
