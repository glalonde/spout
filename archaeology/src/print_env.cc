#include "base/logging.h"

int main(int argc, char** argv, char** envp) {
  for (char** env = envp; *env != 0; env++) {
    LOG(INFO) << *env;
  }
  return 0;
}
