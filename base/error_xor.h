#pragma once
#include <variant>
#include "base/logging.h"

// Wrapper for std::string so ErrorOr can hold strings.
class ErrorMessage {
 public:
  ErrorMessage() = default;
  ErrorMessage(std::string&& message) : message_(std::move(message)) {}
  const std::string& message() const {
    return message_;
  }

 private:
  std::string message_;
};

// Holds either an ErrorMessage xor the desired type (not both)
template <class T>
class ErrorXor {
 public:
  ErrorXor(const T& value) : error_xor_value_(value) {}
  ErrorXor(T&& value) : error_xor_value_(std::move(value)) {}
  ErrorXor(ErrorMessage&& error) : error_xor_value_(std::move(error)) {}

  ErrorXor<T>& operator=(const ErrorXor<T>&) = default;
  ErrorXor<T>& operator=(ErrorXor<T>&&) = default;

  // Uninitialized error message
  ErrorXor() = default;

  operator bool() const {
    return std::holds_alternative<T>(error_xor_value_);
  }

  const ErrorMessage* ErrorOrNull() const {
    return std::get_if<ErrorMessage>(&error_xor_value_);
  }

  const ErrorMessage& ErrorOrDie() const {
    const auto* maybe_error = ErrorOrNull();
    CHECK(maybe_error);
    return *maybe_error;
  }

  const ErrorMessage* ValueOrNull() const {
    return std::get_if<T>(error_xor_value_);
  }

  ErrorMessage* ValueOrNull() {
    return std::get_if<T>(error_xor_value_);
  }

  const T& ValueOrDie() const {
    const auto* maybe_value = ValueOrNull(error_xor_value_);
    CHECK(maybe_value);
    return *maybe_value;
  }

  T&& ValueOrDie() {
    const auto* maybe_value = ValueOrNull(error_xor_value_);
    CHECK(maybe_value);
    return std::move(*maybe_value);
  }

 private:
  std::variant<ErrorMessage, T> error_xor_value_;
};
