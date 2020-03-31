

#ifndef COLOR_UTILS_H_
#define COLOR_UTILS_H_

static const int A_CHANNEL_SIZE = 2*4;
static const int A_CHANNEL_PREFIX = 0;

static const int B_CHANNEL_SIZE = 2*4;
static const int B_CHANNEL_PREFIX = A_CHANNEL_SIZE;

static const int G_CHANNEL_SIZE = 2*4;
static const int G_CHANNEL_PREFIX = B_CHANNEL_SIZE + A_CHANNEL_SIZE;

static const int R_CHANNEL_SIZE = 2*4;
static const int R_CHANNEL_PREFIX = G_CHANNEL_SIZE + B_CHANNEL_SIZE + A_CHANNEL_SIZE;


static const uint32_t A_CHANNEL_MASK = ((1 << A_CHANNEL_SIZE) - 1) << (A_CHANNEL_PREFIX);
static const uint32_t R_CHANNEL_MASK = ((1 << R_CHANNEL_SIZE) - 1) << (R_CHANNEL_PREFIX);
static const uint32_t B_CHANNEL_MASK = ((1 << B_CHANNEL_SIZE) - 1) << (B_CHANNEL_PREFIX);
static const uint32_t G_CHANNEL_MASK = ((1 << G_CHANNEL_SIZE) - 1) << (G_CHANNEL_PREFIX);

struct ColorChannels {
  uint8_t red;
  uint8_t green;
  uint8_t blue;
  uint8_t alpha;
};

inline static uint8_t GetChannel(pixel_t color, int channel_size, int prefix_size) {
  color |= ((1 << channel_size) - 1) << (prefix_size);
  color >>= prefix_size;
  return (uint8_t)color;
}

inline static ColorChannels GetChannels(pixel_t color) {
  ColorChannels col;
  col.red = (uint8_t)((color >> R_CHANNEL_PREFIX) & (((1 << R_CHANNEL_SIZE) - 1)));
  col.green = (uint8_t)((color >> G_CHANNEL_PREFIX) & (((1 << G_CHANNEL_SIZE) - 1)));
  col.blue = (uint8_t)((color >> B_CHANNEL_PREFIX) & (((1 << B_CHANNEL_SIZE) - 1)));
  return col;
}

inline static uint8_t SetChannel(pixel_t color, uint8_t channel_val, int channel_size, int prefix_size) {
  // Clear existing bits
  color &= ~((1 << channel_size) - 1) << (prefix_size);
  // Set the new ones
  color |= channel_val << (prefix_size);
  
  return color;
}

inline static pixel_t SetChannels(ColorChannels channels) {
  pixel_t color = 0x0;

  color |= channels.red;
  color <<= R_CHANNEL_SIZE;
  color |= channels.green;
  color <<= G_CHANNEL_SIZE;
  color |= channels.blue;
  color <<= B_CHANNEL_SIZE;
  color |= ((uint8_t)0xff);
  return color;
}

inline uint8_t Lerp(double percent, int start_point, int end_point) {
  return (uint8_t)(start_point + ((end_point - start_point)*percent));
}

inline uint32_t ColorLerp(double percent, uint32_t start_color, uint32_t end_color) {
  ColorChannels start_channels = GetChannels(start_color);
  ColorChannels end_channels = GetChannels(end_color);
  uint8_t r_new = Lerp(percent, start_channels.red, end_channels.red);
  uint8_t g_new = Lerp(percent, start_channels.green, end_channels.green);
  uint8_t b_new = Lerp(percent, start_channels.blue, end_channels.blue);
  ColorChannels new_channels = {.red = r_new, .green = g_new, .blue = b_new};
  pixel_t new_color = SetChannels(new_channels);
  return new_color;
}

inline void InitColorMap(pixel_t start_color, pixel_t end_color, pixel_t* color_map, int num_types) {
  // First spot is reserved for having no meaning.
  color_map[0] = 0x0;
  for (int i = 0; i < num_types; i ++) {
    color_map[i + 1] = ColorLerp(i/((float)(num_types - 1)), start_color, end_color);
  }
}

inline void InitColorMap2(pixel_t start_color, pixel_t end_color, pixel_t* color_map, int num_types) {
  for (int i = 0; i < num_types; i ++) {
    color_map[i] = ColorLerp(i/((float)(num_types - 1)), start_color, end_color);
  }
}

inline void InitThreeColorMap(pixel_t color1, pixel_t color2, pixel_t color3, int grade1, int grade2, pixel_t* color_map) {
  for (int i = 0; i < grade1; i++) {
    color_map[i] = ColorLerp(i/((float)(grade1 - 1)), color1, color2);
  }
  for (int i = 0; i < grade2; i++) {
    color_map[grade1 + i] = ColorLerp(i/((float)(grade2 - 1)), color2, color3);
  }
}



#endif
