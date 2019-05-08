#include "gpu_particles/game_window.h"
#include "base/logging.h"

GameWindow::GameWindow(int window_width, int window_height)
    : window_width_(window_width), window_height_(window_height) {
  Init();
}

GameWindow::~GameWindow() {
  SDL_GL_DeleteContext(gl_context_);
  SDL_DestroyWindow(window_);
  SDL_Quit();
}

bool GameWindow::IsFullScreen() {
  return SDL_GetWindowFlags(window_) & SDL_WINDOW_FULLSCREEN_DESKTOP;
}

void GameWindow::ToggleFullScreen() {
  if (IsFullScreen()) {
    SDL_SetWindowFullscreen(window_, 0);
  } else {
    SDL_SetWindowFullscreen(window_, SDL_WINDOW_FULLSCREEN_DESKTOP);
  }
}

void GameWindow::SwapWindow() {
  SDL_GL_SwapWindow(window_);
}

void GameWindow::HandleEvents() {
  while (SDL_PollEvent(&event_)) {
    UpdateControllerInput(event_, &input_);
    UpdateWindowState(event_);
  }
}

void GameWindow::UpdateInput(ControllerInput* input) {
  while (SDL_PollEvent(&event_)) {
    UpdateControllerInput(event_, input);
  }
}

void GameWindow::Init() {
  SDL_Init(SDL_INIT_EVERYTHING);
  uint32_t window_flags = SDL_WINDOW_SHOWN | SDL_WINDOW_OPENGL;
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
  SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
  SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);
  window_ = SDL_CreateWindow("Spout", SDL_WINDOWPOS_UNDEFINED,
                             SDL_WINDOWPOS_UNDEFINED, window_width_,
                             window_height_, window_flags);
  gl_context_ = SDL_GL_CreateContext(window_);
  if (!gl_context_) {
    LOG(FATAL) << "Couldn't create OpenGL context, error: " << SDL_GetError();
  }
  if (!gladLoadGL()) {
    LOG(FATAL) << "Something went wrong.";
  }
  SDL_GL_SetSwapInterval(1);
  SDL_ShowCursor(0);
}

void GameWindow::UpdateWindowState(const SDL_Event& event) {
  switch (event.type) {
    case SDL_WINDOWEVENT: {
      switch (event.window.event) {
        case SDL_WINDOWEVENT_RESIZED: {
          glViewport(0, 0, event.window.data1, event.window.data2);
          break;
        }
        default: {
          break;
        }
      }
    }
    default: {
      break;
    }
  }
}
