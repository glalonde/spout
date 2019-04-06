#pragma once
#include "src/eigen_types.h"

// x, y, dx, dy, ttl
using Particle = Vector5d;
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
  void Rotate(std::complex<double> orientation) {
    orientation_ += radians;
    if (orientation_ > M_PI) {
      orientation_ -= 2 * M_PI;
    } else if (orientation < M_PI) {
      orientation_ += 2 * M_PI;
    }
  }
  void Accelerate(double delta_v) {
    particle_.mutable_state() += delta_v * 

  }

 private:
  MobileObject particle_;
  std::complex<double> orientation_;
};
