#ifndef EMITTER_H_
#define EMITTER_H_

#include "vec.h"
#include "particle.h"
#include "ring_buffer.h"
#include "packed_bits.h"

class Emitter {
  public:
    // Emitter properties
    Vec angle;
    double spread;
    vec_dimension min_speed, max_speed, speed_range;

    // Emission properties
    RingBuffer<Particle> particles;

    double emission_progress;
    double emission_period;
    double particle_life;
  
    // Colors
    pixel_t color_map[256];

    Emitter(double spread, vec_dimension min_speed, vec_dimension max_speed, int emits_s, double particle_life);
    ~Emitter();
    void EmitOverTime(double time, Vec pos, Vec velocity_offset, double angle);
    void Emit(int num_emissions, Vec pos, Vec velocity_offset, double angle);
    void Reset();
};

#endif
