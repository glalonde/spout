#pragma once
#include "src/eigen_types.h"

struct IntParticle {
  Vector2<uint32_t> position;
  Vector2<int32_t> velocity;
  float ttl;
  uint32_t padding;
};
