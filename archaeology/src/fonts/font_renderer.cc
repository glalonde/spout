#include "src/fonts/font_renderer.h"
#include "src/fonts/font8x8.h"

namespace font_rendering {
const uint8_t* GetBasicFontBitmap(const char letter) {
  return font8x8_basic[static_cast<int>(letter)];
}

}  // namespace font_rendering
