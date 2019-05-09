#pragma once
#include "gpu_particles/game_parameters.h"
#include "gpu_particles/gl_particle.h"
#include "graphics/opengl.h"

class Emitter {
 public:
  Emitter(EmitterParameters params);

  void EmitOverTime(float dt, Vector2u32 start_pos, Vector2u32 end_pos);

  int num_particles() const {
    return num_particles_;
  }

  GLuint particle_ssbo() const {
    return particle_ssbo_;
  }

 private:
  void InitEmitterShader();
  void MakeParticleBuffer();
  void Emit(int num_emitted, Vector2u32 start_pos, Vector2u32 end_pos);

  // Emitter constants
  EmitterParameters params_;

  // Computed from emitter constants
  float emission_period_;
  int num_particles_;

  // Shader handle
  GLuint emitter_program_;
  GLuint particle_ssbo_;

  // State
  float emission_progress_;
  int write_index_;
};
