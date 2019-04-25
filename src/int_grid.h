#pragma once
#include <climits>
#include <cstdint>

template <class R>
static constexpr R Bitmask(unsigned int const onecount) {
  return static_cast<R>(-(onecount != 0)) &
         (static_cast<R>(-1) >> ((sizeof(R) * CHAR_BIT) - onecount));
}

template <int Bits>
static constexpr uint32_t kHighResMask = Bitmask<uint32_t>(Bits);

template <int Bits>
uint32_t GetLowRes(uint32_t v) {
  return v >> Bits;
}

template <int Bits>
uint32_t GetHighRes(uint32_t v) {
  return v & kHighResMask<Bits>;
}

template <int Bits>
uint32_t SetLowRes(uint32_t v) {
  return v << Bits;
}

template <int Bits>
uint32_t SetHighRes(uint32_t v) {
  return v | kHighResMask<Bits>;
}

template <class R, int Bits>
static R kAnchor = Bitmask<uint32_t>(((sizeof(R) * CHAR_BIT) - Bits) - 1);

template <class R, int Bits>
static R kCellSize = Bitmask<uint32_t>(Bits) + 1;

template <class R, int Bits>
static R kHalfCell = kCellSize<R, Bits> / 2;
