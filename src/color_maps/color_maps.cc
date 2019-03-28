#include "src/color_maps/color_maps.h"

Vector3f GetMappedColor3f(ColorMap map, double p) {
  switch (map) {
    case kParula:
      return EvaluateColorMap<ColorMap::kParula>(p);
    case kMagma:
      return EvaluateColorMap<ColorMap::kMagma>(p);
    case kInferno:
      return EvaluateColorMap<ColorMap::kInferno>(p);
    case kPlasma:
      return EvaluateColorMap<ColorMap::kPlasma>(p);
    case kViridis:
      return EvaluateColorMap<ColorMap::kViridis>(p);
    default:
      return EvaluateColorMap<ColorMap::kParula>(p);
  }
}
