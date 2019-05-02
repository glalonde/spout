#ifndef PARTICLE_H_
#define PARTICLE_H_
#include "mobile_object.h"
#include "constants.h"
#include "motion_constants.h"
#include "ring_buffer.h"
#include "types.h"
#include "screen.h"
#include "color_utils.h"
#include "packed_grid/int_vec.h"

struct Particle : MobileObject {
  public:
    cell_t variation;
    pixel_t* color_map;

    double time_to_live;
    double initial_life;

    Particle();
    Particle(double time_to_live, Vec pos, Vec velocity, cell_t variation, pixel_t* color_map);

    virtual void Reset(double time_to_live, Vec pos, Vec velocity, cell_t variation);
    virtual void Update(double time);
};

// Default constructor
inline Particle::Particle() : MobileObject() {
  time_to_live = 0;
  initial_life = 0;
}

// Main constructor
inline Particle::Particle(double time_to_live,
                          Vec pos,
                          Vec velocity,
                          cell_t variation,
                          pixel_t* color_map) : MobileObject(time_to_live > 0, pos, pos, velocity),
                                                variation(variation),
                                                color_map(color_map),
                                                time_to_live(time_to_live),
                                                initial_life(time_to_live) {
  if (time_to_live > 0) {
    SetActive();
  } else {
    SetInactive();
  }
}

// Almost the same as main constructor, but doesn't change the color map pointer.
inline void Particle::Reset(double time_to_live, Vec pos, Vec velocity, cell_t variation) {
  this->variation = variation;
  this->time_to_live = time_to_live;
  this->initial_life = time_to_live;
  if (time_to_live > 0) {
    MobileObject::Reset(true, pos, pos, velocity);
    SetActive();
  } else {
    MobileObject::Reset(false, pos, pos, velocity);
    SetInactive();
  }
}

inline void Particle::Update(double time) {
  time_to_live -= time;
  if (time_to_live > 0) {
    velocity = Vec_add(velocity, Vec_multiply(PARTICLE_GRAVITY, time));
    prev_pos = pos;
    pos = Vec_add(pos, Vec_multiply(velocity, time));
  } else {
    SetInactive();
  }
}

#endif
