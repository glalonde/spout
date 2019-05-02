#ifndef TYPES_H_
#define TYPES_H_
#include <stdint.h>
#include <assert.h>


typedef uint32_t pixel_t;
typedef uint8_t counter_t;

enum cell_types {
  EMPTY = 0,
  TERRAIN = 1,
  SHIP = 2,
  TAIL = 3,
  GRAIN = 4
}; typedef uint8_t cell_t;

namespace TERRAIN_TYPES {
  static const cell_t EMPTY = 0;
  static const cell_t EDGE = 249;
  static const cell_t FULL = 250;
}

namespace COLORS {
  static const uint32_t RED = 0xFF0000FF;
  static const uint32_t DARK_RED = 0x422020FF;
  static const uint32_t ORANGERED = 0xE34234FF;
  static const uint32_t GREEN = 0x198c19FF;
  static const uint32_t DARK_GREEN = 0x005900FF;
  static const uint32_t BLUE = 0x0000FFFF;
  static const uint32_t LIGHT_BLUE = 0x6960ECFF;
  static const uint32_t BLACK = 0x000000FF;
  static const uint32_t GREY = 0x323232FF;
  static const uint32_t DARK_GREY = 0x212121FF;
  static const uint32_t DARKER_GREY = 0x101010FF;
  static const uint32_t WHITE = 0xFFFFFFFF;
  static const uint32_t PAPAYA = 0xFFF0D0FF;
  static const uint32_t MAGENTA = 0xFF00FFFF;
  static const uint32_t YELLOW = 0xFFFF00FF;
  static const uint32_t DARK = 0x2B1B17FF;
  static const uint32_t TRANSPARENT_BLACK = 0x00000000;
}
/*
typedef uint8_t packed_cell_t;

static const packed_cell_t total_bits = 8;
static const packed_cell_t type_bits = 2;
static const packed_cell_t draw_shift = total_bits - 1;
static const packed_cell_t type_shift = draw_shift - type_bits;
static const packed_cell_t num_types = (packed_cell_t)(1 << type_bits);
static const packed_cell_t num_sub_types = (packed_cell_t)(1 << (type_shift));
static const packed_cell_t draw_mask = (packed_cell_t)(1 << 7);
static const packed_cell_t type_mask = (packed_cell_t)((num_types - 1) << (type_shift));
static const packed_cell_t sub_type_mask = (packed_cell_t)(num_sub_types - 1);


inline static packed_cell_t PackType(bool do_draw, packed_cell_t type, packed_cell_t sub_type) {
  assert(type < num_types);
  assert(sub_type < num_sub_types);
  packed_cell_t draw_word = do_draw << draw_shift;
  packed_cell_t type_word = type << type_shift;
  packed_cell_t sub_type_word = sub_type;

  return (draw_word | type_word | sub_type_word);
}

// If this is greater than 0, draw.
inline static bool DoDraw(packed_cell_t pack) {
  return pack > draw_mask - 1;
}

inline static packed_cell_t GetSubType(packed_cell_t pack) {
  return pack & sub_type_mask;
}

inline static packed_cell_t GetType(packed_cell_t pack) {
  return (pack & type_mask) >> type_shift;
}

inline static packed_cell_t DecrementCounter(packed_cell_t pack) {
  if (pack & sub_type_mask) {
    pack--;
  }
  return pack;
}
 */
#endif
