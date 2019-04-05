#include <stdio.h>
#include <array>

#include "base/format.h"
#include "src/check_opengl_errors.h"
#include "src/image_viewer/image_viewer.h"
#include "src/load_shader.h"
#include "src/opengl.h"

class ImageViewer::Impl {
 public:
  Impl(int window_width, int window_height)
      : window_size_(window_width, window_height) {
    InitWindow();
    InitRenderShader();
    MakeTexture();
  }

  ~Impl() {
    glDeleteTextures(1, &tex_handle_);
    SDL_GL_DeleteContext(gl_context_);
    SDL_DestroyWindow(window_);
    SDL_Quit();
  }

  bool IsFullScreen() {
    return SDL_GetWindowFlags(window_) & SDL_WINDOW_FULLSCREEN_DESKTOP;
  }

  void ToggleFullScreen() {
    if (IsFullScreen()) {
      SDL_SetWindowFullscreen(window_, 0);
    } else {
      SDL_SetWindowFullscreen(window_, SDL_WINDOW_FULLSCREEN_DESKTOP);
    }
  }

  void SetWindowSize(int width, int height) {
    CHECK_GE(width, 0);
    CHECK_GE(height, 0);
    if (width != window_size_[0] || height != window_size_[1]) {
      window_size_ = {width, height};
      SDL_SetWindowSize(window_, width, height);
    }
  }

  void SetTextureSize(int width, int height) {
    CHECK_GE(width, 0);
    CHECK_GE(height, 0);
    if (width != image_.cols() || height != image_.rows()) {
      image_.resize(height, width);
    }
  }

  // Mutate the image data, then call update.
  Image<PixelType::RGBAU8>* data() {
    return &image_;
  }

  void SetDataChanged() {
    data_changed_ = true;
  }

  ControllerInput Update() {
    HandleEvents(&input_);
    if (data_changed_) {
      UpdateTexture();
    }
    if (ShouldRedraw()) {
      Render();
    }
    return input_;
  }

  void Render() {
    glClearColor(1.0, 1.0, 1.0, 1.0);
    glClear(GL_COLOR_BUFFER_BIT);
    glUseProgram(render_program_);
    glBindTexture(GL_TEXTURE_2D, tex_handle_);
    glBindVertexArray(vertex_array_);
    glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);
    SDL_GL_SwapWindow(window_);
    data_changed_ = false;
    window_changed_ = false;
  }

  void HandleEvents(ControllerInput* input) {
    while (SDL_PollEvent(&event_)) {
      UpdateControllerInput(event_, input);
      UpdateWindowState(event_);
    }
  }

 private:
  bool ShouldRedraw() {
    return data_changed_ || window_changed_;
  }

  void UpdateWindowState(const SDL_Event& event) {
    switch (event.type) {
      case SDL_WINDOWEVENT: {
        switch (event.window.event) {
          case SDL_WINDOWEVENT_EXPOSED: {
            window_changed_ = true;
            break;
          }
          case SDL_WINDOWEVENT_RESIZED: {
            window_changed_ = true;
            glViewport(0, 0, event.window.data1, event.window.data2);
            break;
          }
          default: {
            break;
          }
        }
      }
      default: {
        break;
      }
    }
  }

  void InitWindow() {
    SDL_Init(SDL_INIT_EVERYTHING);
    uint32_t window_flags =
        SDL_WINDOW_SHOWN | SDL_WINDOW_OPENGL | SDL_WINDOW_BORDERLESS;
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK,
                        SDL_GL_CONTEXT_PROFILE_CORE);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);
    window_ = SDL_CreateWindow("Image", SDL_WINDOWPOS_UNDEFINED,
                               SDL_WINDOWPOS_UNDEFINED, window_size_[0],
                               window_size_[1], window_flags);
    gl_context_ = SDL_GL_CreateContext(window_);
    if (!gl_context_) {
      LOG(FATAL) << "Couldn't create OpenGL context, error: " << SDL_GetError();
    }
    if (!gladLoadGL()) {
      LOG(FATAL) << "Something went wrong.";
    }
    SDL_GL_SetSwapInterval(1);
    SDL_ShowCursor(0);
    PrintContextAttributes();
  }

  void PrintContextAttributes() {
    int r_size;
    int g_size;
    int b_size;
    int a_size;
    int d_size;
    SDL_GL_GetAttribute(SDL_GL_RED_SIZE, &r_size);
    SDL_GL_GetAttribute(SDL_GL_GREEN_SIZE, &g_size);
    SDL_GL_GetAttribute(SDL_GL_BLUE_SIZE, &b_size);
    SDL_GL_GetAttribute(SDL_GL_ALPHA_SIZE, &a_size);
    SDL_GL_GetAttribute(SDL_GL_DEPTH_SIZE, &d_size);
    LOG(INFO) << FormatString(
        "Red: %s, Green: %s, Blue: %s, Alpha: %s, Depth: %s", r_size, g_size,
        b_size, a_size, d_size);
  }

  void InitRenderShader() {
    render_program_ = glCreateProgram();
    GLuint vert = LoadShader("src/image_viewer/shader.vert", GL_VERTEX_SHADER);
    GLuint frag =
        LoadShader("src/image_viewer/shader.frag", GL_FRAGMENT_SHADER);
    glAttachShader(render_program_, vert);
    glAttachShader(render_program_, frag);
    glBindFragDataLocation(render_program_, 0, "out_color");
    CHECK(CheckGLErrors());
    LinkProgram(render_program_);
    CHECK(CheckGLErrors());

    // Generate buffers
    glGenBuffers(1, &vertex_buffer_);
    glGenVertexArrays(1, &vertex_array_);
    glGenBuffers(1, &element_buffer_);

    // Vertex layout
    struct Vertex {
      Vector2f position;
      Vector2f texture_coordinate;
    };

    // Vertex data
    std::array<Vertex, 4> vertices;
    // Bottom right
    vertices[0].position = {1.0f, -1.0f};
    vertices[0].texture_coordinate = {1.0f, 0.0f};
    // Top right
    vertices[1].position = {1.0f, 1.0f};
    vertices[1].texture_coordinate = {1.0f, 1.0f};
    // Top left
    vertices[2].position = {-1.0f, 1.0f};
    vertices[2].texture_coordinate = {0.0f, 1.0f};
    // Bottom left
    vertices[3].position = {-1.0f, -1.0f};
    vertices[3].texture_coordinate = {0.0f, 0.0f};

    // Element data
    std::array<GLuint, 6> indices = {0, 1, 3, 1, 2, 3};

    // Bind vertex array.
    glBindVertexArray(vertex_array_);
    // Bind vertex buffer
    glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer_);
    glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices.data(),
                 GL_STATIC_DRAW);
    // Bind element buffer
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element_buffer_);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(indices), indices.data(),
                 GL_STATIC_DRAW);

    // Setup pointer to the position data
    GLint pos_ptr = glGetAttribLocation(render_program_, "position");
    glVertexAttribPointer(pos_ptr, 2, GL_FLOAT, GL_FALSE, sizeof(Vertex),
                          reinterpret_cast<void*>(0));
    glEnableVertexAttribArray(pos_ptr);

    // Setup pointer to the texture coordinate data
    GLint tex_coord_ptr =
        glGetAttribLocation(render_program_, "in_texture_coordinate");
    glVertexAttribPointer(tex_coord_ptr, 2, GL_FLOAT, GL_FALSE, sizeof(Vertex),
                          reinterpret_cast<void*>(sizeof(Vector2f)));
    glEnableVertexAttribArray(tex_coord_ptr);
    CHECK(CheckGLErrors());

    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    glEnable(GL_BLEND);
  }

  void MakeTexture() {
    glGenTextures(1, &tex_handle_);
    glBindTexture(GL_TEXTURE_2D, tex_handle_);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
    CHECK(CheckGLErrors());
  }

  void UpdateTexture() {
    glBindTexture(GL_TEXTURE_2D, tex_handle_);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, image_.cols(), image_.rows(), 0,
                 GL_RGBA, GL_UNSIGNED_BYTE, image_.data());
    CHECK(CheckGLErrors());
  }

  Vector2i window_size_;
  Image<PixelType::RGBAU8> image_;
  bool data_changed_;
  bool window_changed_;

  ControllerInput input_;
  SDL_Event event_;
  SDL_Window* window_;
  SDL_GLContext gl_context_;

  GLuint render_program_;
  GLuint vertex_buffer_;
  GLuint vertex_array_;
  GLuint element_buffer_;

  GLuint tex_handle_;
};

ImageViewer::ImageViewer(int width, int height)
    : impl_(std::make_unique<ImageViewer::Impl>(width, height)) {}

ImageViewer::~ImageViewer() = default;

void ImageViewer::SetWindowSize(int width, int height) {
  impl_->SetWindowSize(width, height);
}

void ImageViewer::SetTextureSize(int width, int height) {
  impl_->SetTextureSize(width, height);
}

Image<PixelType::RGBAU8>* ImageViewer::data() {
  return impl_->data();
}

void ImageViewer::SetDataChanged() {
  impl_->SetDataChanged();
}

bool ImageViewer::IsFullScreen() {
  return impl_->IsFullScreen();
}

void ImageViewer::ToggleFullScreen() {
  impl_->ToggleFullScreen();
}

ControllerInput ImageViewer::Update() {
  return impl_->Update();
}
