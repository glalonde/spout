#include "gpu_particles/gl_emitter.h"
#include "graphics/check_opengl_errors.h"
#include "graphics/load_shader.h"

Emitter::Emitter(float emission_rate, float min_life, float max_life)
    : emission_rate_(emission_rate),
      min_life_(min_life),
      max_life_(max_life),
      emission_period_(1.0 / emission_rate_),
      num_particles_(static_cast<int>(std::ceil(emission_rate_ * max_life_))),
      emission_progress_(0) {
  InitEmitterShader();
  MakeParticleBuffer();
}

void Emitter::EmitOverTime(float dt, Vector2u32 start_pos, Vector2u32 end_pos) {
  emission_progress_ += dt;
  if (emission_progress_ > emission_period_) {
    const int num_emissions =
        static_cast<int>(emission_progress_ / emission_period_);
    emission_progress_ -= num_emissions * emission_period_;
    Emit(num_emissions, start_pos, end_pos);
  }
  return;
}

void Emitter::InitEmitterShader() {
  emitter_program_ = glCreateProgram();
  GLuint compute_shader =
      LoadShader("gpu_particles/shaders/emitter.cs", GL_COMPUTE_SHADER);
  glAttachShader(emitter_program_, compute_shader);
  LinkProgram(emitter_program_);
  CHECK(CheckGLErrors());
}

void Emitter::MakeParticleBuffer() {
  const int buffer_size = num_particles_ * (sizeof(IntParticle));
  glGenBuffers(1, &particle_ssbo_);
  glBindBuffer(GL_SHADER_STORAGE_BUFFER, particle_ssbo_);
  glBufferData(GL_SHADER_STORAGE_BUFFER, buffer_size, NULL, GL_DYNAMIC_COPY);
  CHECK(CheckGLErrors());
}

void Emitter::Emit(int num_emitted, Vector2u32 start_pos, Vector2u32 end_pos) {
  // Execute the emitter shader
  glUseProgram(emitter_program_);
  glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 0 /* bind index */,
                   particle_ssbo_);
  glUniform1i(glGetUniformLocation(emitter_program_, "start_index"),
              write_index_);
  glUniform1i(glGetUniformLocation(emitter_program_, "num_emitted"),
              num_emitted);
  glUniform1f(glGetUniformLocation(emitter_program_, "ttl_min"), min_life_);
  glUniform1f(glGetUniformLocation(emitter_program_, "ttl_max"), max_life_);
  glUniform2ui(glGetUniformLocation(emitter_program_, "start_position"),
               start_pos.x(), start_pos.y());
  glUniform2ui(glGetUniformLocation(emitter_program_, "end_position"),
               end_pos.x(), end_pos.y());
  const int group_size = std::min(num_particles_, 512);
  const int num_groups = num_particles_ / group_size;
  glad_glDispatchCompute(num_groups, 1, 1);
  glMemoryBarrier(GL_SHADER_STORAGE_BARRIER_BIT);
  glUseProgram(0);
  CHECK(CheckGLErrors());
  write_index_ = (write_index_ + num_emitted) % num_particles_;
}
