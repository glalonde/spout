#include "constants.h"
#include "motion_constants.h"

#include "spout_game.h"
#include "screen.h"
#include "environment.h"
#include "palette.h"
#include <stdlib.h>
#include <iostream>
#include <cstring>

SpoutGame::SpoutGame(Screen<counter_t>* screen, Screen<counter_t>* particle_screen) : screen(screen),
                                                                                   particle_screen(particle_screen),
                                                                                   game_state(BEFORE_GAME) {
}

void SpoutGame::HandleInput(ControllerInput* input) {
  if (input->reset) {
    game_state = BEFORE_GAME;
    env.Reset();
    screen->Reset();
    particle_screen->Reset();
  }
  
  switch (game_state) {
    case DURING_GAME:
      if (input->pause) {
        game_state = GAME_PAUSED;
      } else {
        env.HandleInput(input);
      }
      break;
    case BEFORE_GAME:
      if (input->up) {
        game_state = DURING_GAME;
      }
      break;
    case GAME_OVER:
      if (input->reset) {
        game_state = BEFORE_GAME;
        env.Reset();
        screen->Reset();
      }
      break;
    case GAME_PAUSED:
      if (input->up) {
        game_state = DURING_GAME;
      }
    default:
      break;
  }
}

void SpoutGame::Update(double interval) {
  // Clear the screen now, because update does the drawing.
  switch (game_state) {
    case DURING_GAME:
      screen->Clear();
      particle_screen->Clear();
      if (env.Update(interval)) {
        game_state = GAME_OVER;
      }
      break;
    case GAME_OVER:
      screen->Clear();
      particle_screen->Clear();
      if (env.Update(interval)) {
        game_state = GAME_OVER;
      }
      break;
    default:
      break;
  }
}

void SpoutGame::Draw() {

  char string_buff[16] = {0};
  switch (game_state) {
    case DURING_GAME:
      sprintf(string_buff, "%f", env.time_left);
      screen->DrawString(string_buff, GRID_WIDTH/2, 0, TerrainPalette::GREEN, true);
      break;
    case BEFORE_GAME:
      //screen->DrawString("BUCKLE UP", GRID_WIDTH/2, GRID_HEIGHT/2, COLORS::RED, true);
      screen->DrawString("1", GRID_WIDTH/2, GRID_HEIGHT/2, TerrainPalette::RED, true);
      break;
    case GAME_OVER:
      //screen->DrawString("YOU'RE DONE!", GRID_WIDTH/2, GRID_HEIGHT/2, COLORS::RED, true);
      screen->DrawString("2", GRID_WIDTH/2, GRID_HEIGHT/2, TerrainPalette::RED, true);
      break;
    case GAME_PAUSED:
      //screen->DrawString("PAUSED", GRID_WIDTH/2, GRID_HEIGHT/2, COLORS::RED, true);
      screen->DrawString("3", GRID_WIDTH/2, GRID_HEIGHT/2, TerrainPalette::RED, true);
    default:
      break;
  }
  sprintf(string_buff, "%d", env.score);
  screen->DrawString(string_buff, 5, 0, TerrainPalette::WHITE, false);
}
