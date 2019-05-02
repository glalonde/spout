#ifndef PACKED_BITS_H_
#define PACKED_BITS_H_
#include <stdint.h>   
#include <assert.h>

typedef uint32_t block32_t;
typedef uint64_t block64_t;

static const int BLOCK_SIZE = 64;
class PackedBits {
public:
  int num_bits;
  int num_blocks;
  uint8_t* data;

  PackedBits() : num_bits(-1), num_blocks(-1), data(NULL) {};

  PackedBits(int num_bits) : num_bits(num_bits), num_blocks((num_bits + BLOCK_SIZE - 1) / BLOCK_SIZE), data((uint8_t*)(new block64_t[num_blocks])) {
    ClearAll();
  }
  ~PackedBits() {
    delete[] data;
  }

  inline void ClearAll() {
    memset(data, 0, sizeof(block64_t)*num_blocks);
  }

  inline void SetAll() {
    memset(data, 0xFF, sizeof(block64_t)*num_blocks);
  }

  static inline uint8_t GetByteMask(int bit_offset) {
    assert(bit_offset >= 0 && bit_offset < 8);
    return (uint8_t)0x1 << (bit_offset);
  }

  inline void SetBit(int bit_index) {
    assert(bit_index >= 0 && bit_index < num_bits);
    data[bit_index / 8] |= GetByteMask(bit_index % 8);
  }

  inline void ClearBit(int bit_index) {
    assert(bit_index >= 0 && bit_index < num_bits);
    data[bit_index / 8] &= ~GetByteMask(bit_index % 8);
  }

  inline bool CheckBit(int bit_index) {
    assert(bit_index >= 0 && bit_index < num_bits);
    return data[bit_index / 8] & GetByteMask(bit_index % 8);
  }

  // If block is clear(0) returns the index of the next bit outside this block
  // if it is not clear, it returns -1;
  inline block64_t GetBlock(int bit_index) {
    assert(bit_index >= 0 && bit_index < num_bits);
    return ((block64_t*)data)[bit_index / BLOCK_SIZE];
  }

  inline int PopCount() {
    int count = 0;
    for (int i = 0; i < num_bits; i += BLOCK_SIZE) {
      block64_t block = GetBlock(i);
      if (block != 0) {
        for (unsigned int j = 0; j < sizeof(block64_t); j++) {
          uint8_t byte = ((uint8_t*)&block)[j];
          if (((uint8_t*)&block)[j] != 0) {
            for (int k = 0; k < 8; k++) {
              if (byte & PackedBits::GetByteMask(k)) {
                if (CheckBit(i + 8*j + k)) {
                  count++;
                }
              }
            }
          }
        }
      }
    }
    return count;
  }
};
  


#endif // PACKED_BITS_H_
