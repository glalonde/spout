#pragma once

//
// class Foo {
//  public:
//   NO_COPY_NO_MOVE_NO_ASSIGN(Foo)
// ...
#define NO_COPY_NO_MOVE_NO_ASSIGN(Classname) \
  Classname(const Classname&) = delete;      \
  void operator=(const Classname&) = delete; \
  Classname(Classname&&) = delete;           \
  void operator=(Classname&&) = delete;
