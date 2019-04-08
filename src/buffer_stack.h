#pragma once

#include "base/logging.h"
#include "src/eigen_types.h"


template <class BufferType>
class BufferStack {
 public:
  using Scalar = typename BufferType::Scalar;
  BufferStack(int rows /* each buffer should have the same shape */)
      : rows_(rows) {}

  void EmplaceBuffer(BufferType buffer) {
    DCHECK_EQ(buffer.rows(), rows_);
    buffers_.emplace_back(std::move(buffer));
  }

  int GetBufferIndex(int row, int* buffer_offset) const {
    int buffer_index = row / rows_;
    *buffer_offset = row - buffer_index * rows_;
    return buffer_index;
  }

  const BufferType& GetBuffer(int global_row, int* local_row) const {
    return buffers_[GetBufferIndex(global_row, local_row)];
  }

  const std::vector<BufferType>& buffers() const {
    return buffers_;
  }

  const Scalar& operator()(int row, int col) const {
    int local_row;
    const auto& buffer = GetBuffer(row, &local_row);
    return buffer(local_row, col);
  }

 private:
  int rows_;
  std::vector<BufferType> buffers_;
};
