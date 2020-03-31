#pragma once
#include <variant>
#include "base/format.h"
#include "base/logging.h"

// Wrapper for std::string so ErrorOr can hold strings.
class ErrorMessage {
 public:
  ErrorMessage() = default;
  ErrorMessage(std::string&& message) : message_(std::move(message)) {}
  ErrorMessage(const std::string_view& message) : message_(message) {}
  const std::string& message() const {
    return message_;
  }

 private:
  std::string message_;
};

// TODO(glalonde) add thread id
#define TraceError(message) \
  ErrorMessage(FormatString("%s:%s] %s", __FILE__, __LINE__, message))

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

  ErrorMessage* ErrorOrNull() {
    return std::get_if<ErrorMessage>(&error_xor_value_);
  }

  const ErrorMessage& ErrorOrDie() const {
    const auto* maybe_error = ErrorOrNull();
    CHECK(maybe_error);
    return *maybe_error;
  }

  const T* ValueOrNull() const {
    return std::get_if<T>(&error_xor_value_);
  }

  T* ValueOrNull() {
    return std::get_if<T>(&error_xor_value_);
  }

  const T& ValueOrDie() const {
    const auto* maybe_value = ValueOrNull();
    CHECK(maybe_value);
    return *maybe_value;
  }

  T&& ValueOrDie() {
    auto* maybe_value = ValueOrNull();
    CHECK(maybe_value);
    return std::move(*maybe_value);
  }

 private:
  std::variant<ErrorMessage, T> error_xor_value_;
};

// Holds either an ErrorMessage or void
// Default constructor is an uninitialized Error
// To represent no error return ErrorXor<void>::NoError();
template <>
class ErrorXor<void> {
 public:
  ErrorXor() : maybe_error_(ErrorMessage()) {}
  ErrorXor(ErrorMessage&& error) : maybe_error_(std::move(error)) {}
  ErrorXor(const ErrorXor& error) = default;
  ErrorXor<void>& operator=(const ErrorXor<void>&) = default;
  ErrorXor<void>& operator=(ErrorXor<void>&&) = default;

  static ErrorXor NoError() {
    ErrorXor out;
    out.maybe_error_.reset();
    return out;
  }

  operator bool() const {
    return !maybe_error_;
  }

  const ErrorMessage* ErrorOrNull() const {
    if (maybe_error_) {
      return &(*maybe_error_);
    }
    return nullptr;
  }

  const ErrorMessage& ErrorOrDie() const {
    const auto* maybe_error = ErrorOrNull();
    CHECK(maybe_error);
    return *maybe_error;
  }

 private:
  std::optional<ErrorMessage> maybe_error_;
};

// String formatting
namespace std {
inline std::ostream& operator<<(std::ostream& os,
                                const ErrorMessage& error_message) {
  os << error_message.message();
  return os;
}

template <class T>
inline std::ostream& operator<<(std::ostream& os,
                                const ErrorXor<T>& maybe_value) {
  if (maybe_value) {
    os << *maybe_value.ValueOrNull();
  } else {
    os << *maybe_value.ErrorOrNull();
  }
  return os;
}
template <>
inline std::ostream& operator<<(std::ostream& os,
                                const ErrorXor<void>& maybe_value) {
  if (maybe_value) {
    os << "";
  } else {
    os << *maybe_value.ErrorOrNull();
  }
  return os;
}
}  // namespace std
