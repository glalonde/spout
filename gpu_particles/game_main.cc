#include <array>
#include <memory>
#include "base/format.h"
#include "base/init.h"
#include "base/wall_timer.h"
#include "gpu_particles/game_window.h"
#include "gpu_particles/gl_emitter.h"
#include "graphics/check_opengl_errors.h"
#include "graphics/load_shader.h"
#include "graphics/opengl.h"
#include "src/color_maps/color_maps.h"
#include "src/controller_input.h"
#include "src/demo_utils.h"
#include "src/eigen_types.h"
#include "src/image.h"
#include "src/int_grid.h"
#include "src/random.h"
#include "src/so2.h"

DEFINE_bool(debug, false, "Debug mode");
DEFINE_int32(color_map_index, 0, "Color map index, see color_maps.h");
DEFINE_double(damage_rate, 1.0, "Damage rate");
DEFINE_double(dt, .016, "Simulation rate");
DEFINE_int32(pixel_size, 4, "Pixel size");
DEFINE_double(particle_speed, 250.0, "Particle speed");
DEFINE_double(emission_rate, 250.0, "Particle emission rate");
DEFINE_double(min_life, 1.0, "Min particle life");
DEFINE_double(max_life, 5.0, "Max particle life");

static constexpr int kMantissaBits = 14;

static constexpr int32_t kDenseWall = 1000;

class ParticleSim {
 public:
  ParticleSim(int window_width, int window_height, int grid_width,
              int grid_height)
      : window_(window_width, window_height),
        grid_dims_(grid_width, grid_height) {
    Init();
  }

  ~ParticleSim() {
  }

  ControllerInput Update(const float dt) {
    window_.HandleEvents();
    const auto& input = window_.input();
    if (input.up) {
      Vector2u32 emit_position = Vector2u32::Constant(
          SetLowRes<kMantissaBits>(kAnchor<uint32_t, kMantissaBits>));
      emit_position += ((grid_dims_ / 2) * cell_size_).cast<uint32_t>();
      emitter_->EmitOverTime(dt, emit_position - Vector2u32(cell_size_, 0) * 30,
                             emit_position + Vector2u32(cell_size_, 0) * 30);
    }
    UpdateParticleSimulation(dt);
    UpdateShipSimulation(dt, Vector2f(0.f, -250 * cell_size_));
    Render();
    return input;
  }

  void UpdateParticleSimulation(float dt) {
    // Clear the density counter texture
    uint32_t clear_color = 0;
    glClearTexImage(particle_tex_handle_, 0, GL_RED_INTEGER, GL_UNSIGNED_INT,
                    &clear_color);
    CHECK(CheckGLErrors());
    // Update particle states
    glUseProgram(particle_program_);
    glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 0 /* bind index */,
                     particle_ssbo_);
    glUniform1f(glGetUniformLocation(particle_program_, "dt"), dt);
    glUniform1i(glGetUniformLocation(particle_program_, "anchor"),
                kAnchor<uint32_t, kMantissaBits>);
    glUniform1i(glGetUniformLocation(particle_program_, "buffer_width"),
                grid_dims_[0]);
    glUniform1i(glGetUniformLocation(particle_program_, "buffer_height"),
                grid_dims_[1]);
    glUniform1f(glGetUniformLocation(particle_program_, "damage_rate"),
                FLAGS_damage_rate);
    glUniform1i(glGetUniformLocation(particle_program_, "kMantissaBits"),
                kMantissaBits);
    const int group_size = std::min(num_particles_, 512);
    const int num_groups = num_particles_ / group_size;
    glad_glDispatchCompute(num_groups, 1, 1);
    glMemoryBarrier(GL_SHADER_STORAGE_BARRIER_BIT);
    CHECK(CheckGLErrors());
  }

  void UpdateShipSimulation(float dt, Vector2f acceleration) {
    glUseProgram(particle_program_);
    glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 0 /* bind index */, ship_ssbo_);
    glUniform1f(glGetUniformLocation(particle_program_, "dt"), dt);
    glUniform1i(glGetUniformLocation(particle_program_, "anchor"),
                kAnchor<uint32_t, kMantissaBits>);
    glUniform1i(glGetUniformLocation(particle_program_, "buffer_width"),
                grid_dims_[0]);
    glUniform1i(glGetUniformLocation(particle_program_, "buffer_height"),
                grid_dims_[1]);
    glUniform1f(glGetUniformLocation(particle_program_, "damage_rate"),
                FLAGS_damage_rate);
    glUniform1i(glGetUniformLocation(particle_program_, "kMantissaBits"),
                kMantissaBits);
    glad_glDispatchCompute(1, 1, 1);
    glMemoryBarrier(GL_SHADER_STORAGE_BARRIER_BIT);
    CHECK(CheckGLErrors());

    glUseProgram(ship_program_);
    glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 0 /* bind index */, ship_ssbo_);
    glUniform1f(glGetUniformLocation(ship_program_, "dt"), dt);
    glUniform2f(glGetUniformLocation(ship_program_, "acceleration"),
                acceleration.x(), acceleration.y());
    glad_glDispatchCompute(1, 1, 1);
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

    window_.SwapWindow();
    CHECK(CheckGLErrors());
  }

  Vector<IntParticle, Eigen::Dynamic> ReadParticleBuffer() {
    CHECK(CheckGLErrors());
    const int buffer_size = num_particles_ * (sizeof(IntParticle));
    glBindBuffer(GL_SHADER_STORAGE_BUFFER, particle_ssbo_);
    CHECK(CheckGLErrors());
    void* buffer_ptr = glMapBufferRange(GL_SHADER_STORAGE_BUFFER, 0,
                                        buffer_size, GL_MAP_READ_BIT);
    CHECK(CheckGLErrors());
    Eigen::Map<Vector<IntParticle, Eigen::Dynamic>> points(
        reinterpret_cast<IntParticle*>(buffer_ptr), num_particles_);
    Vector<IntParticle, Eigen::Dynamic> copied = points;
    CHECK(CheckGLErrors());
    glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);
    CHECK(CheckGLErrors());
    return copied;
  }

  IntParticle ReadShipBuffer() {
    const int buffer_size = 1 * (sizeof(IntParticle));
    glBindBuffer(GL_SHADER_STORAGE_BUFFER, ship_ssbo_);
    CHECK(CheckGLErrors());
    void* buffer_ptr = glMapBufferRange(GL_SHADER_STORAGE_BUFFER, 0,
                                        buffer_size, GL_MAP_READ_BIT);
    CHECK(CheckGLErrors());
    IntParticle out = *reinterpret_cast<IntParticle*>(buffer_ptr);
    glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);
    CHECK(CheckGLErrors());
    return out;
  }

 private:
  void Init() {
    std::mt19937 rando(0);
    Image<int32_t> level_buffer(grid_dims_.y(), grid_dims_.x());
    level_buffer.setZero();
    MakeLevel(&rando, &level_buffer);
    LOG(INFO) << level_buffer.maxCoeff();

    IntParticle initial_ship;
    initial_ship.position = Vector2u32::Constant(
        SetLowRes<kMantissaBits>(kAnchor<uint32_t, kMantissaBits>));
    initial_ship.position += (grid_dims_ / 2 * cell_size_).cast<uint32_t>();
    initial_ship.velocity.setZero();
    initial_ship.ttl = 100000;
    initial_ship.padding = 0;

    InitEmitter();
    num_particles_ = emitter_->num_particles();
    particle_ssbo_ = emitter_->particle_ssbo();

    MakeShipBuffer(initial_ship);
    MakeTerrainTexture(level_buffer);
    MakeDensityTexture();
    MakeParticleColorTable();
    MakeTerrainColorTable();
    InitBresenhamShader();
    InitShipShader();
    InitRenderShader();
    LOG(INFO) << "Finished init";
  }

  void MakeLevel(std::mt19937* gen, Image<int32_t>* level_buffer) {
    AddNoise(kDenseWall, .5, gen, level_buffer);
    AddAllWalls(kDenseWall, level_buffer);
  }

  void InitEmitter() {
    emitter_ = std::make_unique<Emitter>(FLAGS_emission_rate, FLAGS_min_life,
                                         FLAGS_max_life);
  }

  void MakeShipBuffer(const IntParticle& init) {
    const int buffer_size = 1 * (sizeof(IntParticle));
    glGenBuffers(1, &ship_ssbo_);
    glBindBuffer(GL_SHADER_STORAGE_BUFFER, ship_ssbo_);
    glBufferData(GL_SHADER_STORAGE_BUFFER, buffer_size, NULL, GL_DYNAMIC_COPY);
    GLint buf_mask = GL_MAP_WRITE_BIT | GL_MAP_INVALIDATE_BUFFER_BIT;
    void* buffer_ptr =
        glMapBufferRange(GL_SHADER_STORAGE_BUFFER, 0, buffer_size, buf_mask);
    IntParticle* ship_ptr = reinterpret_cast<IntParticle*>(buffer_ptr);
    *ship_ptr = init;
    glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);
    CHECK(CheckGLErrors());
    LOG(INFO) << "Made ship buffer";
  }

  void InitBresenhamShader() {
    particle_program_ = glCreateProgram();
    GLuint compute_shader =
        LoadShader("gpu_particles/shaders/bresenham.cs", GL_COMPUTE_SHADER);
    glAttachShader(particle_program_, compute_shader);
    LinkProgram(particle_program_);
    CHECK(CheckGLErrors());
  }

  void InitShipShader() {
    ship_program_ = glCreateProgram();
    GLuint compute_shader =
        LoadShader("gpu_particles/shaders/ship.cs", GL_COMPUTE_SHADER);
    glAttachShader(ship_program_, compute_shader);
    LinkProgram(ship_program_);
    CHECK(CheckGLErrors());
  }

  void InitRenderShader() {
    render_program_ = glCreateProgram();
    GLuint vert =
        LoadShader("gpu_particles/shaders/shader.vert", GL_VERTEX_SHADER);
    GLuint frag = LoadShader("gpu_particles/shaders/color_map_texture.frag",
                             GL_FRAGMENT_SHADER);
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

  GameWindow window_;
  const Vector2i grid_dims_;
  const int cell_size_ = kCellSize<uint32_t, kMantissaBits>;
  int num_particles_;


  // Particle data
  GLuint particle_ssbo_;
  GLuint ship_ssbo_;

  // Compute shaders
  GLuint particle_program_;
  GLuint ship_program_;
  std::unique_ptr<Emitter> emitter_;

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
  int window_width = 1440;
  int window_height = 900;
  int pixel_size = FLAGS_pixel_size;
  int grid_width = window_width / pixel_size;
  int grid_height = window_height / pixel_size;

  ParticleSim sim(window_width, window_height, grid_width, grid_height);

  double dt = FLAGS_dt;
  ControllerInput input;
  while (!input.quit) {
    input = sim.Update(dt);
  }
}

int main(int argc, char* argv[]) {
  Init(argc, argv);
  TestLoop();
  return 0;
}
