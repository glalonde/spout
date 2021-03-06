#pragma once

#include "base/logging.h"
#include "src/eigen_types.h"

// Represents, stores and multiplexes a stack of buffers, meaning literally
// stacked vertically in 2D to accomodate scrolling functionality.
//
// TODO(glalonde) Allow deleting old buffers
template <class BufferType>
class BufferStack {
 public:
  using Scalar = typename BufferType::Scalar;
  BufferStack(int rows /* each buffer should have the same shape */, int cols)
      : rows_(rows), cols_(cols) {}

  void EmplaceBuffer(BufferType buffer) {
    DCHECK_EQ(buffer.rows(), rows_);
    DCHECK_EQ(buffer.cols(), cols_);
    buffers_.emplace_back(std::move(buffer));
  }

  // The buffer index and local row offset corresponding to a global row
  int GetBufferIndex(int row, int* buffer_offset) const {
    int buffer_index = row / rows_;
    *buffer_offset = row - buffer_index * rows_;
    return buffer_index;
  }

  // The the actual buffer reference and the local row offset.
  const BufferType& GetBuffer(int global_row, int* local_row) const {
    return buffers_[GetBufferIndex(global_row, local_row)];
  }

  BufferType& GetMutableBuffer(int global_row, int* local_row) {
    return buffers_[GetBufferIndex(global_row, local_row)];
  }

  // Const accessor
  const std::vector<BufferType>& buffers() const {
    return buffers_;
  }

  // Unsafe multiplexed coefficient access
  const Scalar& operator()(int row, int col) const {
    int local_row;
    const auto& buffer = GetBuffer(row, &local_row);
    return buffer(local_row, col);
  }

  Scalar& operator()(int row, int col) {
    int local_row;
    auto& buffer = GetMutableBuffer(row, &local_row);
    return buffer(local_row, col);
  }

  int rows() const {
    return buffers_.size() * rows_;
  }

  int cols() const {
    return cols_;
  }

 private:
  int rows_;
  int cols_;
  std::vector<BufferType> buffers_;
};
