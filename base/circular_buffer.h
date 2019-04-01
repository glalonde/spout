#pragma once
#include <optional>
#include <vector>

template <class T>
class CircularBuffer {
 public:
  CircularBuffer(int capacity) : write_index_(0) {
    data_.reserve(capacity);
  }

  // Return a ptr to the element that will be overwritten in the next Push
  // call. Returns nullptr if the container is not at capacity yet (and thus
  // nothing will be overwritten)
  const T* NextOverwritten() {
    if (data_.size() < data_.capacity()) {
      return nullptr;
    } else {
      return &data_[write_index_];
    }
  }

  void Push(T value) {
    if (data_.size() < data_.capacity()) {
      // Nothing overwritten
      data_.emplace_back(std::move(value));
    } else {
      data_[write_index_] = std::move(value);
      ++write_index_;
      if (write_index_ >= data_.capacity()) {
        write_index_ = 0;
      }
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
