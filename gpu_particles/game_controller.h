#pragma once
#include <memory>
#include "gpu_particles/game_parameters.h"
#include "gpu_particles/game_window.h"
#include "gpu_particles/gl_emitter.h"
#include "gpu_particles/gl_particle.h"
#include "graphics/opengl.h"
#include "src/image.h"
#include "src/int_grid.h"
#include "src/random.h"

static constexpr int kMantissaBits = 14;

static constexpr int32_t kDenseWall = 1000;

class ParticleSim {
 public:
  ParticleSim(int window_width, int window_height, GameParameters params);

  ~ParticleSim();

  ControllerInput Update(const float dt);

  void UpdateParticleSimulation(float dt);

  void UpdateShipSimulation(float dt, Vector2f acceleration);

  void Render();

  Vector<IntParticle, Eigen::Dynamic> ReadParticleBuffer();

  IntParticle ReadShipBuffer();

 private:
  void Init();

  void MakeLevel(std::mt19937* gen, Image<int32_t>* level_buffer);

  void InitEmitter();

  void MakeShipBuffer(const IntParticle& init);

  void InitBresenhamShader();

  void InitShipShader();

  void InitRenderShader();

  void MakeTerrainTexture(const Image<int32_t>& terrain_data);

  void MakeDensityTexture();

  // Make a color gradient texture to sample.
  void MakeParticleColorTable(const int n_steps = 256);

  // Make a color gradient texture to sample.
  void MakeTerrainColorTable(const int n_steps = 256);

  GameWindow window_;
  GameParameters params_;
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
