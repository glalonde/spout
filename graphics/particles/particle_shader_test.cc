#include <array>
#include "base/init.h"
#include "base/wall_timer.h"
#include "graphics/check_opengl_errors.h"
#include "graphics/load_shader.h"
#include "graphics/opengl.h"
#include "src/controller_input.h"
#include "src/eigen_types.h"
#include "src/int_grid.h"
#include "src/random.h"

struct IntParticle {
  Vector2<uint32_t> position;
  Vector2<int32_t> velocity;
};

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

  void UpdateTexture(float dt) {
    glUseProgram(compute_program_);
    glUniform1f(glGetUniformLocation(compute_program_, "dt"), dt);
    glUniform1i(glGetUniformLocation(compute_program_, "anchor"),
                kAnchor<uint32_t, 8>);
    const int group_size = std::min(num_particles_, 512);
    const int num_groups = num_particles_ / group_size;
    glad_glDispatchComputeGroupSizeARB(num_groups, 1, 1, group_size, 1, 1);
    glMemoryBarrier(GL_SHADER_STORAGE_BARRIER_BIT);
    CHECK(CheckGLErrors());
  }

  void Render(float dt) {
    UpdateTexture(dt);
    SDL_GL_SwapWindow(window_);
    CHECK(CheckGLErrors());
    ReadParticleBuffer();
  }

  void UpdateInput(ControllerInput* input) {
    while (SDL_PollEvent(&event_)) {
      UpdateControllerInput(event_, input);
    }
  }

  void ReadParticleBuffer() {
    CHECK(CheckGLErrors());
    const int buffer_size = num_particles_ * (sizeof(IntParticle));
    glBindBuffer(GL_SHADER_STORAGE_BUFFER, particle_ssbo_);
    CHECK(CheckGLErrors());
    LOG(INFO) << "Reading buffer size: " << buffer_size;
    void* buffer_ptr = glMapBufferRange(GL_SHADER_STORAGE_BUFFER, 0,
                                        buffer_size, GL_MAP_READ_BIT);
    CHECK(CheckGLErrors());
    Eigen::Map<Vector<IntParticle, Eigen::Dynamic>> points(
        reinterpret_cast<IntParticle*>(buffer_ptr), num_particles_);
    CHECK(CheckGLErrors());
    LOG(INFO) << points[0].position.transpose();
    LOG(INFO) << points[0].velocity.transpose();
    glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);
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

    MakeParticleBuffer();
    InitComputeShader();
    LOG(INFO) << "Finished init";
  }

  void InitComputeShader() {
    compute_program_ = glCreateProgram();
    GLuint compute_shader =
        LoadShader("graphics/particles/bresenham.cs", GL_COMPUTE_SHADER);
    glAttachShader(compute_program_, compute_shader);
    LinkProgram(compute_program_);
    glUseProgram(compute_program_);
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
    const int buffer_size = num_particles_ * (sizeof(IntParticle));
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
    Eigen::Map<Vector<IntParticle, Eigen::Dynamic>> points(
        reinterpret_cast<IntParticle*>(buffer_ptr), num_particles_);
    points[0].position = {5, 6};
    points[0].velocity = {62, 124};
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
  GLuint compute_program_;
};

int main(int argc, char* argv[]) {
  Init(argc, argv);
  SDLContainer sdl({600, 600}, {100, 100}, 1);
  ControllerInput input;
  TimePoint previous;
  while (!input.quit) {
    const TimePoint current = ClockType::now();
    const float dt = ToSeconds<float>(current - previous);
    sdl.UpdateInput(&input);
    sdl.Render(dt);
    previous = current;
  }
  return 0;
}
