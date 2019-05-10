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

static constexpr uint32_t SetValues(int mantissa_bits, uint32_t integral,
                                    uint32_t fractional) {
  return (integral << mantissa_bits) | fractional;
}

template <class R>
static constexpr R Anchor(int mantissa_bits) {
  return Bitmask<uint32_t>(((sizeof(R) * CHAR_BIT) - mantissa_bits) - 1);
}

template <class R, int MantissaBits>
static R kAnchor = Anchor<R>(MantissaBits);

template <class R>
static constexpr R CellSize(int mantissa_bits) {
  return Bitmask<uint32_t>(mantissa_bits) + 1;
}

template <class R, int MantissaBits>
static R kCellSize = CellSize<R>(MantissaBits);

template <class R>
static constexpr R HalfCellSize(int mantissa_bits) {
  return CellSize<R>(mantissa_bits) >> 1;
}

template <class R, int MantissaBits>
static R kHalfCellSize = HalfCellSize<R>(MantissaBits);
