#pragma once

#include <array>
#include "src/eigen_types.h"

enum ColorMap { kParula, kMagma, kInferno, kPlasma, kViridis };
static const std::array<ColorMap, 5> kAllColorMaps = {kParula, kMagma, kInferno,
                                                      kPlasma, kViridis};

// To make a color map, specialize this function for your map.
template <ColorMap Map>
Vector3f EvaluateColorMap(double p);

Vector3f GetMappedColor3f(ColorMap map, double p);
