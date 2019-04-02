#include <stdio.h>
#include <iostream>

#include "base/init.h"
#include "src/fonts/font8x8.h"

DEFINE_string(text, "ayy lmao", "Text to render.");

void Render(const uint8_t* bitmap) {
  for (int x = 0; x < 8; x++) {
    for (int y = 0; y < 8; y++) {
      const int set = bitmap[x] & 1 << y;
      std::cout << (set ? 'X' : ' ');
    }
    std::cout << std::endl;
  }
}

int RenderLine(const std::string& text) {
  const auto& text_chars = text.c_str();
  for (int i = 0; i < text.size(); ++i) {
    Render(font8x8_basic[static_cast<int>(text_chars[i])]);
  }
  return 0;
}

int main(int argc, char** argv) {
  Init(argc, argv);
  RenderLine(FLAGS_text);
}
