#ifndef RING_BUFFER_H_
#define RING_BUFFER_H_
#include <cstddef>
#include <iostream>

#include "packed_bits.h"

template<class type>
struct RingBuffer {
  int size;
  int write_index;
  type* buffer;

  RingBuffer();
  RingBuffer(int max_elements);
  ~RingBuffer();

  void Insert(const type& to_insert);
  type* GetNext();
};

template<class type>
inline RingBuffer<type>::RingBuffer(void) : size(0),
                                            write_index(-1),
                                            buffer(NULL) { }

template<class type>
inline RingBuffer<type>::RingBuffer(int max_elements) : size(max_elements),
                                                        write_index(size - 1),
                                                        buffer(new type[max_elements]) { }

template<class type>
inline RingBuffer<type>::~RingBuffer() {
  delete[] buffer;
}

template<class type>
inline void RingBuffer<type>::Insert(const type& to_insert) {
  if (++write_index >= size) { write_index = 0; }
  buffer[write_index] = to_insert;
}

template<class type>
inline type* RingBuffer<type>::GetNext() {
  if (++write_index >= size) { write_index = 0; }
  return &buffer[write_index];
}

#endif
