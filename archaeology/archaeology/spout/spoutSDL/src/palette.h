#ifndef PALETTE_H_
#define PALETTE_H_
#include <assert.h>
#include "types.h"
#include "color_utils.h"

class TexturePalette {
public:
  const static int NUM_COLORS = 256;
  const static uint8_t EMPTY = 255;
};



class TerrainPalette : public TexturePalette {
public:
  const static uint8_t UNDAMAGED_TERRAIN = TERRAIN_TYPES::FULL;
  const static uint8_t ALMOST_DEAD_TERRAIN = TERRAIN_TYPES::EMPTY + 1;
  const static uint8_t DEAD_TERRAIN = TERRAIN_TYPES::EMPTY;

  const static uint8_t MAGENTA = 251;
  const static uint8_t GREEN = 253;
  const static uint8_t WHITE = 252;
  const static uint8_t RED = 254;

  static void GetPalette(pixel_t* pal) {
    for (int i = 0; i < NUM_COLORS; i++) {
      pal[i] = COLORS::TRANSPARENT_BLACK;
    }

    // Transisition between undamaged to damaged terrain
    InitColorMap2(COLORS::ORANGERED, COLORS::DARK_GREY, &pal[ALMOST_DEAD_TERRAIN], UNDAMAGED_TERRAIN - ALMOST_DEAD_TERRAIN);
    pal[UNDAMAGED_TERRAIN] = COLORS::DARK_GREY;
    pal[MAGENTA] = COLORS::MAGENTA;
    pal[GREEN] = COLORS::GREEN;
    pal[WHITE] = COLORS::WHITE;
    pal[RED] = COLORS::RED;
    pal[EMPTY] = COLORS::TRANSPARENT_BLACK;
  };
};

class ParticlePalette : public TexturePalette {
public:
  const static uint8_t FIRST_GRADIENT = 0;
  const static uint8_t SECOND_GRADIENT = 20;
  const static uint8_t THIRD_GRADIENT = 40;

  static void GetPalette(pixel_t* pal) {
    for (int i = 0; i < NUM_COLORS; i++) {
      pal[i] = COLORS::TRANSPARENT_BLACK;
    }

    InitColorMap2(COLORS::TRANSPARENT_BLACK, COLORS::LIGHT_BLUE, &pal[FIRST_GRADIENT], SECOND_GRADIENT - FIRST_GRADIENT);
    InitColorMap2(COLORS::LIGHT_BLUE, COLORS::GREEN, &pal[SECOND_GRADIENT], THIRD_GRADIENT - SECOND_GRADIENT);
    InitColorMap2(COLORS::GREEN, COLORS::WHITE, &pal[THIRD_GRADIENT], NUM_COLORS - THIRD_GRADIENT);
  };
};


#endif
