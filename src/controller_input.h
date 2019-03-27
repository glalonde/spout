#pragma once
#include <SDL2/SDL.h>

struct ControllerInput {
  bool quit = false;
  bool up = false;
  bool down = false;
  bool left = false;
  bool right = false;
  bool reset = false;
  bool pause = false;
};

void UpdateControllerInput(const SDL_Event& event, ControllerInput* input) {
  switch (event.type) {
    case SDL_QUIT:
      input->quit = true;
      break;
    case SDL_KEYDOWN:
    case SDL_KEYUP:
      bool set_val = (event.type == SDL_KEYDOWN);
      switch (event.key.keysym.sym) {
        case SDLK_q:
          input->quit = set_val;
          break;
        case SDLK_UP:
          input->up = set_val;
          break;
        case SDLK_DOWN:
          input->down = set_val;
          break;
        case SDLK_LEFT:
          input->left = set_val;
          break;
        case SDLK_RIGHT:
          input->right = set_val;
          break;
        case SDLK_r:
          input->reset = set_val;
          break;
        case SDLK_p:
          input->pause = set_val;
          break;
      }
  }
}
