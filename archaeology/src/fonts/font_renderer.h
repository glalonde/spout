#pragma once

#include <stdio.h>
#include <Eigen/Geometry>
#include <iostream>

#include "src/eigen_types.h"

namespace font_rendering {
static constexpr int kFontWidth = 8;
static constexpr int kFontHeight = 8;

enum Justification { kLeft, kRight, kCenter };

inline Eigen::AlignedBox<int, 2> GetCharacterBounds(
    const Vector2i& bottom_left,
    const Eigen::AlignedBox<int, 2>& buffer_bounds) {
  Eigen::AlignedBox<int, 2> character_box(
      bottom_left, bottom_left + Vector2i(kFontWidth, kFontHeight));
  character_box = character_box.intersection(buffer_bounds);
  return character_box;
}

template <typename Derived>
inline void RenderCharacter(const uint8_t* bitmap, const Vector2i& bottom_left,
                            const typename Derived::Scalar& set_value,
                            const Eigen::AlignedBox<int, 2>& bounds,
                            Eigen::ArrayBase<Derived>* buffer) {
  const auto character_box = GetCharacterBounds(bottom_left, bounds);
  for (int row = character_box.min().y(); row < character_box.max().y();
       ++row) {
    for (int col = character_box.min().x(); col < character_box.max().x();
         ++col) {
      const int set = bitmap[kFontHeight - (row - bottom_left.y()) - 1] &
                      1 << (col - bottom_left.x());
      if (set) {
        (*buffer)(row, col) = set_value;
      }
    }
  }
}

inline Vector2i GetBottomLeft(const std::string& text,
                              const Vector2i& line_anchor, int kerning,
                              font_rendering::Justification justification) {
  Vector2i character_bottom_left = line_anchor;
  const int text_width =
      text.size() * font_rendering::kFontWidth + (text.size() - 1) * kerning;
  switch (justification) {
    case font_rendering::kLeft:
      break;
    case font_rendering::kRight:
      character_bottom_left.x() -= text_width;
      break;
    case font_rendering::kCenter:
      character_bottom_left.x() -= text_width / 2;
      break;
  }
  return character_bottom_left;
}

const uint8_t* GetBasicFontBitmap(const char letter);

}  // namespace font_rendering

// Anchor is
template <typename Derived>
inline void RenderString(const std::string& text, const Vector2i& anchor,
                         const typename Derived::Scalar& set_value, int kerning,
                         font_rendering::Justification justification,
                         Eigen::ArrayBase<Derived>* buffer) {
  const Eigen::AlignedBox<int, 2> bounds(
      Vector2i::Zero(), Vector2i(buffer->cols(), buffer->rows()));
  auto character_bottom_left =
      GetBottomLeft(text, anchor, kerning, justification);
  const auto& text_chars = text.c_str();
  for (int i = 0; i < text.size(); ++i) {
    font_rendering::RenderCharacter(
        font_rendering::GetBasicFontBitmap(text_chars[i]),
        character_bottom_left, set_value, bounds, buffer);
    character_bottom_left.x() += font_rendering::kFontWidth + kerning;
  }
}
