#pragma once
#include "src/eigen_types.h"
#include "src/so2.h"

// x, y, dx, dy, ttl
using Particle = Vector5d;

// The components that can just be added directly:
// (delta dx, delta dy, delta ttl)
// x and y are updated through bresenham
// as are dx and dy.
using DeltaParticle = Vector3d;

class MobileObject {
 public:
  MobileObject() : MobileObject(Particle::Zero()) {}
  MobileObject(Particle state) : state_(state) {}

  void ApplyDelta(const DeltaParticle& dp) {
    state_.tail<3>() += dp;
  }

  const Particle& state() const {
    return state_;
  }

  Particle* mutable_state() {
    return &state_;
  }

 private:
  Particle state_;
};

class Ship {
 public:
  Ship(MobileObject particle, double orientation)
      : particle_(particle), orientation_(orientation){};
  void Rotate(double orientation) {
    orientation_ *= SO2d(orientation);
  }
  void Accelerate(double delta_v) {
    particle_.mutable_state().segment<2>(2) += delta_v * orientation_.data();
  }

 private:
  MobileObject particle_;
  SO2d orientation_;
};
