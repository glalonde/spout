#include "emitter.h"
#include "constants.h"
#include "color_utils.h"
#include <stdlib.h>
#include <assert.h>

Emitter::Emitter(double spread, vec_dimension min_speed, vec_dimension max_speed, int emits_s, double particle_life) :
      spread(spread),
      min_speed(min_speed),
      max_speed(max_speed),
      speed_range(max_speed-min_speed),
      particles((int)(emits_s*particle_life) + 1),
      emission_progress(0.0),
      emission_period(1.0/emits_s),
      particle_life(particle_life) {

  InitColorMap(COLORS::ORANGERED, COLORS::BLACK, color_map, 256);

  // Initialize all of the particle structs
  for (int i = 0; i < particles.size; i ++) {
    Particle* p = particles.GetNext();
    *p = Particle(0, Vec_make(0, 0), Vec_make(0, 0), 0, color_map);
  }
}

Emitter::~Emitter() {
}

void Emitter::Reset() {
  for (int i = 0; i < particles.size; i ++) {
    particles.GetNext()->Reset(0, Vec_make(0, 0), Vec_make(0, 0), 0);
  }
}

void Emitter::EmitOverTime(double time, Vec pos, Vec velocity_offset, double angle) {
  emission_progress += time;
  if (emission_progress > emission_period) {
    int to_emit = (int)(emission_progress/emission_period);
    emission_progress -= to_emit*emission_period;
    Emit(to_emit, pos, velocity_offset, angle);
  }
}

void Emitter::Emit(int num_emissions, Vec pos, Vec velocity_offset, double angle) {
  for (int i = 0; i < num_emissions; i ++) {
    // Get original direction
    Vec adjusted_velocity = Vec_unit(angle + this->spread*((float)rand()/RAND_MAX - .5));
    // Set speed
    adjusted_velocity = Vec_multiply(adjusted_velocity, this->min_speed + this->speed_range*rand()/RAND_MAX);
    // Adjust for relative motion
    adjusted_velocity = Vec_add(adjusted_velocity, velocity_offset);
    assert(particle_life > 0);
    particles.GetNext()->Reset(particle_life, pos, adjusted_velocity, (int)(((float)rand()/RAND_MAX)*256));
  }
}
