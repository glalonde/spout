#include "base/init.h"
#include "vulkan/game_controller.h"

int main(int argc, char* argv[]) {
  Init(argc, argv);
  GameController app;
  app.Run();
  return EXIT_SUCCESS;
}
