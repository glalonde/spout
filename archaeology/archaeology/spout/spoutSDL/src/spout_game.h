#ifndef SPOUT_GAME_H_
#define SPOUT_GAME_H_

#include "terrain.h"
#include "controller_input.h"
#include "environment.h"
#include "screen.h"

enum GameState { BEFORE_GAME, DURING_GAME, GAME_OVER, GAME_PAUSED };

class SpoutGame {
  public:
    Screen<counter_t>* screen;
    Screen<counter_t>* particle_screen;

    SpoutGame(Screen<counter_t>* screen, Screen<counter_t>* particle_screen);
    void HandleInput(ControllerInput* input);
    void Update(double interval);
    void Draw();
  
    Environment env = Environment(screen, particle_screen);
    GameState game_state;
};


#endif
