#include <array>
#include "base/init.h"
#include "base/wall_timer.h"
#include "graphics/check_opengl_errors.h"
#include "graphics/load_shader.h"
#include "graphics/opengl.h"
#include "src/color_maps/color_maps.h"
#include "src/controller_input.h"
#include "src/eigen_types.h"
#include "src/random.h"

// Use a compute shader to count up the positions of a bunch of particles and
// then draw to a texture in a color corresponding to the density of particles.

class SDLContainer {
 public:
  SDLContainer(const Vector2i& screen_dims, const Vector2i& grid_dims,
               int num_particles)
      : screen_dims_(screen_dims),
        grid_dims_(grid_dims),
        num_particles_(num_particles) {
    Init();
  }

  ~SDLContainer() {
    SDL_GL_DeleteContext(gl_context_);
    SDL_DestroyWindow(window_);
    SDL_Quit();
  }

  void UpdateTexture() {
    uint32_t zero = 0;
    glClearTexImage(tex_handle_, 0, GL_RED_INTEGER, GL_UNSIGNED_INT, &zero);
    CHECK(CheckGLErrors());
    glUseProgram(compute_program_);
    const int group_size = 512;
    const int num_groups = num_particles_ / group_size;
    glad_glDispatchComputeGroupSizeARB(num_groups, 1, 1, group_size, 1, 1);
    glMemoryBarrier(GL_SHADER_STORAGE_BARRIER_BIT);
    CHECK(CheckGLErrors());
  }

  void Render() {
    UpdateTexture();
    glUseProgram(render_program_);

    // Bind in the color lookup table
    glActiveTexture(GL_TEXTURE0 + 1);
    glBindTexture(GL_TEXTURE_1D, color_lut_handle_);
    // glBindSampler(1, linearFiltering);

    glBindVertexArray(vertex_array_);
    glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);
    SDL_GL_SwapWindow(window_);
    CHECK(CheckGLErrors());
  }

  void UpdateInput(ControllerInput* input) {
    while (SDL_PollEvent(&event_)) {
      UpdateControllerInput(event_, input);
    }
  }

  void ReadParticleBuffer() {
    const int buffer_size = num_particles_ * sizeof(Vector2f);
    glBindBuffer(GL_SHADER_STORAGE_BUFFER, particle_ssbo_);
    void* buffer_ptr = glMapBufferRange(GL_SHADER_STORAGE_BUFFER, 0,
                                        buffer_size, GL_MAP_READ_BIT);
    Eigen::Map<Matrix<float, 2, Eigen::Dynamic>> points(
        reinterpret_cast<float*>(buffer_ptr), 2, num_particles_);
    glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);
    CHECK(CheckGLErrors());
  }

  void ReadTexture() {
    glBindTexture(GL_TEXTURE_2D, tex_handle_);
    CHECK(CheckGLErrors());
    Matrix<uint32_t, Eigen::Dynamic, Eigen::Dynamic> out_tex(grid_dims_.x(),
                                                             grid_dims_.y());
    glGetTexImage(GL_TEXTURE_2D, 0, GL_RED_INTEGER, GL_UNSIGNED_INT,
                  out_tex.data());
    CHECK(CheckGLErrors());
  }

 private:
  void Init() {
    SDL_Init(SDL_INIT_EVERYTHING);
    uint32_t window_flags = SDL_WINDOW_SHOWN | SDL_WINDOW_OPENGL;
    window_ =
        SDL_CreateWindow("Test.", screen_dims_.x(), screen_dims_.y(),
                         screen_dims_.x(), screen_dims_.y(), window_flags);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK,
                        SDL_GL_CONTEXT_PROFILE_CORE);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    gl_context_ = SDL_GL_CreateContext(window_);
    if (!gl_context_) {
      LOG(FATAL) << "Couldn't create OpenGL context, error: " << SDL_GetError();
    }
    if (!gladLoadGL()) {
      LOG(FATAL) << "Something went wrong.";
    }
    SDL_GL_SetSwapInterval(1);
    SDL_ShowCursor(0);

    // glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    // glEnable(GL_BLEND);

    MakeParticleBuffer();
    MakeTexture();
    MakeColorTable();
    InitComputeShader();
    InitRenderShader();
    LOG(INFO) << "Finished init";
  }

  void InitComputeShader() {
    compute_program_ = glCreateProgram();
    GLuint compute_shader =
        LoadShader("graphics/particles/draw_particles.cs", GL_COMPUTE_SHADER);
    glAttachShader(compute_program_, compute_shader);
    LinkProgram(compute_program_);
    glUseProgram(compute_program_);
    // glUniform1i(glGetUniformLocation(compute_program_, "counter_texture"), 0);
    CHECK(CheckGLErrors());
  }

  void InitRenderShader() {
    render_program_ = glCreateProgram();
    GLuint vert =
        LoadShader("graphics/particles/shader.vert", GL_VERTEX_SHADER);
    GLuint frag =
        LoadShader("graphics/particles/shader.frag", GL_FRAGMENT_SHADER);
    glAttachShader(render_program_, vert);
    glAttachShader(render_program_, frag);
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
    vertices[0].position = {1.0f, -1.0f};
    vertices[0].texture_coordinate = {1.0f, 1.0f};
    vertices[1].position = {1.0f, 1.0f};
    vertices[1].texture_coordinate = {1.0f, 0.0f};
    vertices[2].position = {-1.0f, 1.0f};
    vertices[2].texture_coordinate = {0.0f, 0.0f};
    vertices[3].position = {-1.0f, -1.0f};
    vertices[3].texture_coordinate = {0.0f, 1.0f};

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
  }

  void MakeTexture() {
    // Make a uint32 texture to hold the counts of each particle in that
    // position.
    glGenTextures(1, &tex_handle_);
    glActiveTexture(GL_TEXTURE0);
    glBindTexture(GL_TEXTURE_2D, tex_handle_);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    const auto format = GL_R32UI;
    Matrix<uint32_t, Eigen::Dynamic, Eigen::Dynamic> out_tex(grid_dims_.x(),
                                                             grid_dims_.y());
    out_tex.setZero();
    glTexImage2D(GL_TEXTURE_2D, 0, format, grid_dims_.x(), grid_dims_.y(), 0,
                 GL_RED_INTEGER, GL_UNSIGNED_INT, out_tex.data());
    CHECK(CheckGLErrors());

    // Because we're also using this tex as an image (in order to write to it),
    // we bind it to an image unit as well
    glBindImageTexture(0, tex_handle_, 0, GL_FALSE, 0, GL_WRITE_ONLY, format);
    CHECK(CheckGLErrors());
  }

  // Make a color gradient texture to sample.
  void MakeColorTable(const int n_steps = 256) {
    // Make a uint32 texture to hold the counts of each particle in that
    // position.
    glGenTextures(1, &color_lut_handle_);
    glActiveTexture(GL_TEXTURE1);  // Does this need to match the shader..?
    glBindTexture(GL_TEXTURE_1D, color_lut_handle_);
    glTexParameteri(GL_TEXTURE_1D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_1D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    const auto format = GL_RGB32F;

    const ColorMap map = ColorMap::kParula;

    // Eigen is column major by default meaning columns are stored contiguously,
    // meaning we want each component of a given color on the same column.
    MatrixXf out_tex(3, n_steps);
    out_tex.setZero();

    // Set the source data for our gradient texture.
    CHECK_GE(n_steps, 2);
    for (int i = 0; i < n_steps; ++i) {
      const double p = static_cast<double>(i) / (n_steps - 1);
      out_tex.col(i) = GetMappedColor3f(map, p);
    }

    glTexImage1D(GL_TEXTURE_1D, 0, format, n_steps, 0, GL_RGB, GL_FLOAT,
                 out_tex.data());
    CHECK(CheckGLErrors());
  }

  void SetRandomPoints(const AlignedBox2f& sample_space,
                       Eigen::Map<Matrix<float, 2, Eigen::Dynamic>> data) {
    std::mt19937 gen(0);
    SetRandomUniform(sample_space.min().x(), sample_space.max().x(), &gen,
                     data.row(0));
    SetRandomUniform(sample_space.min().y(), sample_space.max().y(), &gen,
                     data.row(1));
  }

  void MakeParticleBuffer() {
    const int buffer_size = num_particles_ * sizeof(Vector2f);
    glGenBuffers(1, &particle_ssbo_);
    glBindBuffer(GL_SHADER_STORAGE_BUFFER, particle_ssbo_);
    glBufferData(GL_SHADER_STORAGE_BUFFER, buffer_size, NULL, GL_STATIC_DRAW);
    GLint buf_mask = GL_MAP_WRITE_BIT | GL_MAP_INVALIDATE_BUFFER_BIT;
    void* buffer_ptr =
        glMapBufferRange(GL_SHADER_STORAGE_BUFFER, 0, buffer_size, buf_mask);
    WallTimer timer;
    LOG(INFO) << "Creating " << num_particles_ << " random particles";
    timer.Start();
    Eigen::Map<Matrix<float, 2, Eigen::Dynamic>> points(
        reinterpret_cast<float*>(buffer_ptr), 2, num_particles_);
    SetRandomPoints(AlignedBox2f(Vector2f(0, 0), grid_dims_.cast<float>()),
                    points);
    LOG(INFO) << "Done in: " << timer.ElapsedDuration();
    glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);
    CHECK(CheckGLErrors());
    const GLint particle_buffer_bind_point = 0;
    glBindBufferBase(GL_SHADER_STORAGE_BUFFER, particle_buffer_bind_point,
                     particle_ssbo_);
    timer.Stop();
  }

  const Vector2i screen_dims_;
  const Vector2i grid_dims_;
  const int num_particles_;

  SDL_Event event_;
  SDL_Window* window_;
  SDL_GLContext gl_context_;

  // Particle data
  GLuint particle_ssbo_;

  // Convert particle data to a texture of particle counts
  GLuint tex_handle_;
  GLuint color_lut_handle_;
  GLuint compute_program_;

  // Quad draw
  GLuint render_program_;
  GLuint vertex_buffer_;
  GLuint vertex_array_;
  GLuint element_buffer_;
};

int main(int argc, char* argv[]) {
  Init(argc, argv);
  SDLContainer sdl({600, 600}, {100, 100}, std::pow(2, 15));
  ControllerInput input;
  while (!input.quit) {
    sdl.UpdateInput(&input);
    sdl.Render();
  }
  return 0;
}
