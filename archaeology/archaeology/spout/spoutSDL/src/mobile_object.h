#ifndef MOBILE_OBJECT_H_
#define MOBILE_OBJECT_H_

#include "vec.h"
#include "collision.h"
#include "constants.h"
#include <iostream>
#include <assert.h>

struct MobileObject {
  bool is_active;
  Vec pos;
  Vec prev_pos;
  Vec velocity;

  public:
    MobileObject();
    MobileObject(bool is_active, Vec pos, Vec prev_pos, Vec velocity);
    virtual void Reset(bool is_active, Vec pos, Vec prev_pos, Vec velocity);
    virtual void Update(double time);
    void AddVelocity(Vec velocity);
    void ProcessCollision(Collision* collision);
    virtual void SetActive();
    virtual void SetInactive();
}; typedef struct MobileObject MobileObject;

// Default constructor
inline MobileObject::MobileObject() {
  Reset(false, Vec_make(0,0), Vec_make(0,0), Vec_make(0,0));
}

// Main cosntructor
inline MobileObject::MobileObject(bool is_active, Vec pos, Vec prev_pos, Vec velocity) {
  Reset(is_active, pos, prev_pos, velocity);
}

inline void MobileObject::Reset(bool is_active, Vec pos, Vec prev_pos, Vec velocity) {
  this->is_active = is_active;
  this->pos = pos;
  this->prev_pos = prev_pos;
  this->velocity = velocity;
}

inline void MobileObject::Update(double time) {
  velocity = Vec_add(velocity, Vec_multiply(GRAVITY, time));
  prev_pos = pos;
  pos = Vec_add(pos, Vec_multiply(velocity, time));
}

inline void MobileObject::AddVelocity(Vec velocity) {
  velocity = Vec_add(this->velocity, velocity);
}

inline void MobileObject::ProcessCollision(Collision* collision) {
  //assert(collision->dir_x != 0 || collision->dir_y != 0);
  if (collision->reverse_x) {
    if (collision->dir_x > 0) {
      prev_pos.x = collision->x + collision->dir_x + EPSILON;
    } else {
      prev_pos.x = collision->x - EPSILON;
    }
    prev_pos.y = collision->y + .5;
    assert(collision->dir_y == 0);
    velocity.x *= -COLLISION_ELASTICITY;
    pos.x = prev_pos.x - (pos.x - prev_pos.x);
  } else {
    if (collision->dir_y > 0) {
      prev_pos.y = collision->y + collision->dir_y + EPSILON;
    } else {
      prev_pos.y = collision->y - EPSILON;
    }
    prev_pos.x = collision->x + .5;
    assert(collision->dir_x == 0);
    velocity.y *= -COLLISION_ELASTICITY;
    pos.y = prev_pos.y - (pos.y - prev_pos.y);
  }
}

inline void MobileObject::SetActive() {
  is_active = true;
}

inline void MobileObject::SetInactive() {
  is_active = false;
}

#endif
