#pragma once
#include <optional>
#include <vector>

template <class T>
class CircularBuffer {
 public:
  // Initialized with every value.
  CircularBuffer(int capacity, const T& init_value)
      : write_index_(0), data_(capacity, init_value) {}

  // Return a ptr to the element that will be overwritten in the next Push
  // call. Returns nullptr if the container is not at capacity yet (and thus
  // nothing will be overwritten)
  const T* NextOverwritten() {
    return &data_[write_index_];
  }

  void Push(T value) {
    data_[write_index_] = std::move(value);
    ++write_index_;
    if (write_index_ >= data_.capacity()) {
      write_index_ = 0;
    }
  }

  int Capacity() const {
    return data_.capacity();
  }

  int WriteIndex() {
    return write_index_;
  }

  const std::vector<T>& data() {
    return data_;
  }

 private:
  int write_index_;
  std::vector<T> data_;
};
