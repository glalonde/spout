#include "graphics/opengl.h"
#include "src/controller_input.h"

class GameWindow {
 public:
  GameWindow(int window_width, int window_height);
  ~GameWindow();

  bool IsFullScreen();

  void ToggleFullScreen();

  void SwapWindow();

  void HandleEvents();

  void UpdateInput(ControllerInput* input);

 private:
  void Init();

  void UpdateWindowState(const SDL_Event& event);

  int window_width_;
  int window_height_;

  ControllerInput input_;
  SDL_Event event_;
  SDL_Window* window_;
  SDL_GLContext gl_context_;
};
