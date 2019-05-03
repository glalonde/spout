#include <array>
#include "base/init.h"
#include "base/wall_timer.h"
#include "base/format.h"
#include "graphics/check_opengl_errors.h"
#include "graphics/load_shader.h"
#include "graphics/opengl.h"
#include "src/color_maps/color_maps.h"
#include "src/controller_input.h"
#include "src/eigen_types.h"
#include "src/int_grid.h"
#include "src/random.h"
#include "src/image.h"
#include "src/demo_utils.h"
#include "src/bresenham.h"
#include "src/so2.h"

DEFINE_int32(num_particles, 512, "Number of particles");
DEFINE_bool(debug, false, "Debug mode");
DEFINE_int32(color_map_index, 0, "Color map index, see color_maps.h");
DEFINE_double(damage_rate, 1.0, "Damage rate");

struct Particle {
  Vector2f position;
  Vector2f velocity;
};

static constexpr int32_t kDenseWall = 1000;

class ParticleSim {
 public:
  ParticleSim(int window_width, int window_height, int grid_width,
              int grid_height, int num_particles)
      : window_size_(window_width, window_height),
        grid_dims_(grid_width, grid_height),
        num_particles_(num_particles) {
    Init();
  }

  ~ParticleSim() {
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

  ControllerInput Update(const Duration& dt) {
    HandleEvents(&input_);
    UpdateSimulation(ToSeconds<float>(dt));
    Render();
    return input_;
  }

  void UpdateSimulation(float dt) {
    // Clear the density counter texture
    uint32_t clear_color = 0;
    glClearTexImage(particle_tex_handle_, 0, GL_RED_INTEGER, GL_UNSIGNED_INT,
                    &clear_color);
    CHECK(CheckGLErrors());
    // Update particle states
    glUseProgram(particle_program_);
    glUniform1f(glGetUniformLocation(particle_program_, "dt"), dt);
    glUniform1i(glGetUniformLocation(particle_program_, "buffer_width"),
                grid_dims_[0]);
    glUniform1i(glGetUniformLocation(particle_program_, "buffer_height"),
                grid_dims_[1]);
    glUniform1f(glGetUniformLocation(particle_program_, "damage_rate"),
                FLAGS_damage_rate);
    const int group_size = std::min(num_particles_, 512);
    const int num_groups = num_particles_ / group_size;
    glad_glDispatchComputeGroupSizeARB(num_groups, 1, 1, group_size, 1, 1);
    glMemoryBarrier(GL_SHADER_STORAGE_BARRIER_BIT);
    CHECK(CheckGLErrors());
  }

  void Render() {
    // Clear the screen
    glClearColor(0.0, 0.0, 0.0, 1.0);
    glClear(GL_COLOR_BUFFER_BIT);
    glUseProgram(render_program_);
    glBindVertexArray(vertex_array_);

    // Draw the terrain(signed)
    {
      glUniform1i(glGetUniformLocation(render_program_, "min_value"), 0);
      glUniform1i(glGetUniformLocation(render_program_, "max_value"),
                  kDenseWall);
      glUniform1i(glGetUniformLocation(render_program_, "signed"), true);

      // Active the color texture in unit 0
      glActiveTexture(GL_TEXTURE0);
      glBindTexture(GL_TEXTURE_1D, terrain_color_handle_);

      // Activate the density texture in unit 1
      glActiveTexture(GL_TEXTURE2);
      glBindTexture(GL_TEXTURE_2D, terrain_tex_handle_);

      // Draw the density map
      glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);
    }

    // Draw the particles(unsigned)
    {
      glUniform1i(glGetUniformLocation(render_program_, "min_value"), 0);
      glUniform1i(glGetUniformLocation(render_program_, "max_value"), 15);
      glUniform1i(glGetUniformLocation(render_program_, "signed"), false);
      // Active the color texture in unit 0
      glActiveTexture(GL_TEXTURE0);
      glBindTexture(GL_TEXTURE_1D, particle_color_handle_);

      // Activate the density texture in unit 1
      glActiveTexture(GL_TEXTURE1);
      glBindTexture(GL_TEXTURE_2D, particle_tex_handle_);

      // Draw the density map
      glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);
    }

    SDL_GL_SwapWindow(window_);
    CHECK(CheckGLErrors());
    SDL_GL_SwapWindow(window_);
    CHECK(CheckGLErrors());
  }

  void HandleEvents(ControllerInput* input) {
    while (SDL_PollEvent(&event_)) {
      UpdateControllerInput(event_, input);
      UpdateWindowState(event_);
    }
  }

  void UpdateInput(ControllerInput* input) {
    while (SDL_PollEvent(&event_)) {
      UpdateControllerInput(event_, input);
    }
  }

  Vector<Particle, Eigen::Dynamic> ReadParticleBuffer() {
    CHECK(CheckGLErrors());
    const int buffer_size = num_particles_ * (sizeof(Particle));
    glBindBuffer(GL_SHADER_STORAGE_BUFFER, particle_ssbo_);
    CHECK(CheckGLErrors());
    void* buffer_ptr = glMapBufferRange(GL_SHADER_STORAGE_BUFFER, 0,
                                        buffer_size, GL_MAP_READ_BIT);
    CHECK(CheckGLErrors());
    Eigen::Map<Vector<Particle, Eigen::Dynamic>> points(
        reinterpret_cast<Particle*>(buffer_ptr), num_particles_);
    Vector<Particle, Eigen::Dynamic> copied = points;
    CHECK(CheckGLErrors());
    glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);
    CHECK(CheckGLErrors());
    return copied;
  }
 private:
  void Init() {
    SDL_Init(SDL_INIT_EVERYTHING);
    uint32_t window_flags = SDL_WINDOW_SHOWN | SDL_WINDOW_OPENGL;
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

    std::mt19937 rando(0);
    Image<int32_t> level_buffer(grid_dims_.y(), grid_dims_.x());
    level_buffer.setZero();
    MakeLevel(&rando, &level_buffer);
    LOG(INFO) << level_buffer.maxCoeff();

    MakeParticleBuffer();
    MakeTerrainTexture(level_buffer);
    MakeDensityTexture();
    MakeParticleColorTable();
    MakeTerrainColorTable();
    InitComputeShader();
    InitRenderShader();
    LOG(INFO) << "Finished init";
  }

  void MakeLevel(std::mt19937* gen, Image<int32_t>* level_buffer) {
    AddNoise(kDenseWall, .5, gen, level_buffer);
    AddAllWalls(kDenseWall, level_buffer);
  }

  void UpdateWindowState(const SDL_Event& event) {
    switch (event.type) {
      case SDL_WINDOWEVENT: {
        switch (event.window.event) {
          case SDL_WINDOWEVENT_RESIZED: {
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

  void InitComputeShader() {
    particle_program_ = glCreateProgram();
    GLuint compute_shader =
        LoadShader("graphics/particles/fp_bresenham.cs", GL_COMPUTE_SHADER);
    glAttachShader(particle_program_, compute_shader);
    LinkProgram(particle_program_);
    glUseProgram(particle_program_);
    CHECK(CheckGLErrors());
  }

  void InitRenderShader() {
    render_program_ = glCreateProgram();
    GLuint vert =
        LoadShader("graphics/particles/shader.vert", GL_VERTEX_SHADER);
    GLuint frag =
        LoadShader("graphics/particles/color_map_texture.frag", GL_FRAGMENT_SHADER);
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

    // Turn on alpha blending
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    glEnable(GL_BLEND);
  }

  void MakeTerrainTexture(const Image<int32_t>& terrain_data) {
    // Make a texture to represent the terrain state at each grid cell
    glGenTextures(1, &terrain_tex_handle_);
    glBindTexture(GL_TEXTURE_2D, terrain_tex_handle_);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
    const auto format = GL_R32I;
    CHECK_EQ(terrain_data.rows(), grid_dims_.y());
    CHECK_EQ(terrain_data.cols(), grid_dims_.x());
    glTexImage2D(GL_TEXTURE_2D, 0, format, grid_dims_[0], grid_dims_[1], 0,
                 GL_RED_INTEGER, GL_INT, terrain_data.data());
    CHECK(CheckGLErrors());

    // Because we're also using this tex as an image (in order to write to it),
    // we bind it to an image unit as well
    glBindImageTexture(0, terrain_tex_handle_, 0, GL_FALSE, 0, GL_READ_WRITE,
                       format);
    CHECK(CheckGLErrors());
  }

  void MakeDensityTexture() {
    // Make a uint32 texture to hold the counts of each particle in that
    // position.
    glGenTextures(1, &particle_tex_handle_);
    glBindTexture(GL_TEXTURE_2D, particle_tex_handle_);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
    const auto format = GL_R32UI;
    Matrix<uint32_t, Eigen::Dynamic, Eigen::Dynamic> out_tex(grid_dims_[1],
                                                             grid_dims_[0]);
    out_tex.setZero();
    glTexImage2D(GL_TEXTURE_2D, 0, format, grid_dims_[0], grid_dims_[1], 0,
                 GL_RED_INTEGER, GL_UNSIGNED_INT, out_tex.data());
    CHECK(CheckGLErrors());

    // Because we're also using this tex as an image (in order to write to it),
    // we bind it to an image unit as well
    glBindImageTexture(1, particle_tex_handle_, 0, GL_FALSE, 0, GL_WRITE_ONLY,
                       format);
    CHECK(CheckGLErrors());
  }

  // Make a color gradient texture to sample.
  void MakeParticleColorTable(const int n_steps = 256) {
    // Make a uint32 texture to hold the counts of each particle in that
    // position.
    glGenTextures(1, &particle_color_handle_);
    glBindTexture(GL_TEXTURE_1D, particle_color_handle_);
    glTexParameteri(GL_TEXTURE_1D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_1D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_1D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);

    const auto format = GL_RGB32F;

    const ColorMap map = kAllColorMaps[FLAGS_color_map_index];

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

  // Make a color gradient texture to sample.
  void MakeTerrainColorTable(const int n_steps = 256) {
    // Make a uint32 texture to hold the counts of each particle in that
    // position.
    glGenTextures(1, &terrain_color_handle_);
    glBindTexture(GL_TEXTURE_1D, terrain_color_handle_);
    glTexParameteri(GL_TEXTURE_1D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_1D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_1D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);

    const auto format = GL_RGB32F;

    const ColorMap map = ColorMap::kPlasma;

    // Eigen is column major by default meaning columns are stored contiguously,
    // meaning we want each component of a given color on the same column.
    MatrixXf out_tex(3, n_steps);
    out_tex.setZero();

    // Set the source data for our gradient texture.
    CHECK_GE(n_steps, 2);
    for (int i = 0; i < n_steps; ++i) {
      const double p = 1.0 - static_cast<double>(i) / (n_steps - 1);
      out_tex.col(i) = GetMappedColor3f(map, p);
    }

    glTexImage1D(GL_TEXTURE_1D, 0, format, n_steps, 0, GL_RGB, GL_FLOAT,
                 out_tex.data());
    CHECK(CheckGLErrors());
  }

  void SetRandomPoints(Eigen::Map<Vector<Particle, Eigen::Dynamic>> data) {
    std::mt19937 gen(0);
    auto magnitude_dist = UniformRandomDistribution<float>(0, 500);
    auto angle_dist = UniformRandomDistribution<float>(-M_PI, M_PI);
    for (int i = 0; i < num_particles_; ++i) {
      data[i].position = (grid_dims_).cast<float>() / 2.f;
      data[i].velocity = SO2f(angle_dist(gen)).data() * magnitude_dist(gen);
    }
  }

  void MakeParticleBuffer() {
    const int buffer_size = num_particles_ * (sizeof(Particle));
    glGenBuffers(1, &particle_ssbo_);
    glBindBuffer(GL_SHADER_STORAGE_BUFFER, particle_ssbo_);
    LOG(INFO) << "Making buffer size: " << buffer_size;
    glBufferData(GL_SHADER_STORAGE_BUFFER, buffer_size, NULL, GL_DYNAMIC_COPY);
    GLint buf_mask = GL_MAP_WRITE_BIT | GL_MAP_INVALIDATE_BUFFER_BIT;
    void* buffer_ptr =
        glMapBufferRange(GL_SHADER_STORAGE_BUFFER, 0, buffer_size, buf_mask);
    WallTimer timer;
    LOG(INFO) << "Creating " << num_particles_ << " random particles";
    timer.Start();
    Eigen::Map<Vector<Particle, Eigen::Dynamic>> points(
        reinterpret_cast<Particle*>(buffer_ptr), num_particles_);
    SetRandomPoints(points);
    LOG(INFO) << "Done in: " << timer.ElapsedDuration();
    glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);
    CHECK(CheckGLErrors());
    const GLint particle_buffer_bind_point = 0;
    glBindBufferBase(GL_SHADER_STORAGE_BUFFER, particle_buffer_bind_point,
                     particle_ssbo_);
    timer.Stop();
  }

  const Vector2i window_size_;
  const Vector2i grid_dims_;
  const int num_particles_;

  ControllerInput input_;
  SDL_Event event_;
  SDL_Window* window_;
  SDL_GLContext gl_context_;

  // Particle data
  GLuint particle_ssbo_;

  GLuint particle_program_;

  // Convert particle data to a texture of particle counts
  GLuint particle_tex_handle_;
  GLuint particle_color_handle_;
  GLuint terrain_color_handle_;
  GLuint terrain_tex_handle_;

  // Quad draw
  GLuint render_program_;
  GLuint vertex_buffer_;
  GLuint vertex_array_;
  GLuint element_buffer_;
};
void TestLoop() {
  ParticleSim sim(1440, 900, 576, 360, FLAGS_num_particles);
  sim.ToggleFullScreen();
  ControllerInput input;
  TimePoint previous = ClockType::now();
  while (!input.quit) {
    const TimePoint current = ClockType::now();
    input = sim.Update(current - previous);
    previous = current;
  }
}

int main(int argc, char* argv[]) {
  Init(argc, argv);
  TestLoop();
  return 0;
}
