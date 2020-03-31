#include "environment.h"
#include "types.h"
#include "emitter.h"

#include <math.h>
#include <iostream>
#include <assert.h>
#include <time.h>

Environment::Environment(Screen<counter_t>* screen, Screen<counter_t>* particle_screen) : screen(screen),
                                                                                       particle_screen(particle_screen),
                                                                                       score(ship.pos.y) {
  this->Reset();
}

void Environment::Reset() {
  ship.Reset(Vec_make(GRID_WIDTH/2 + .5, GRID_HEIGHT/2 + .5), M_PI/2);
  terrain.Reset();
  time_left = INITIAL_TIME;
  current_level = 0;
}

void Environment::HandleInput(ControllerInput* input) {
  ship.HandleInput(input);
}

bool Environment::Update(double interval) {
  ship.Update(interval);
  time_left -= interval;
  
  // Scroll upwards and update score if necessary
  // Make sure to notify the terrain so it can swap buffers if necessary
  int ship_screen_height = (int)ship.pos.y - GRID_HEIGHT/2;
  if(ship_screen_height > screen->screen_bottom) {
    score = ship.pos.y;
    int new_level = terrain.GetBuffCount(ship.pos.y);

    screen->SyncHeight(ship_screen_height);
    particle_screen->SyncHeight(ship_screen_height);
    bool level_swap = terrain.SyncHeight(screen->screen_bottom);

    // If we just got to a new level we need to get rid of the particles that were on the previous buffer.
    // They are now out of scope.
    if (level_swap) {
      for (int i = 0; i < particles->size; i ++) {
        if(particles->buffer[i].is_active && particles->buffer[i].pos.y < terrain.bottom_height) {
          particles->buffer[i].SetInactive();
        }
      }
    }

    // If new level, reset the countdown timer
    if (new_level > current_level) {
      time_left += INCREMENTAL_TIME;
      current_level = new_level;
    }
  }

  CheckCollisions(interval);
  terrain.Draw(screen);
  ship.Draw(screen);

  return IsGameOver();
}

void Environment::CheckCollisions(double time) {
  Collision collision;
  if (particles == NULL) return;
  Particle* buff = particles->buffer;
  for (int i = 0; i < particles->size; i ++) {
    if(buff[i].is_active) {
      buff[i].Update(time);
      // Check with the terrain object whether collision happened
      collision = terrain.GetCollisionFast(&buff[i]);
      while (collision.exists) {
        buff[i].ProcessCollision(&collision); 
        terrain.Damage(collision.x, collision.y, Vec_length(buff[i].velocity));
        collision = terrain.GetCollisionFast(&buff[i]);
      }
      if (collision.out_of_scope) {
        buff[i].SetInactive();
      } else if(buff[i].is_active) {
        // Draw the particles
        particle_screen->AddToCell(buff[i].pos.x, buff[i].pos.y, (counter_t)((255/PARTICLE_PER_SPOT * HEAT_APPEARANCE * .2)*buff[i].time_to_live/PARTICLE_LIFE) + 1.0);
      }
    }
  }
}

bool Environment::IsGameOver() {
  // This is only false if the game is already over because of the
  // condition below in the previous round.
  if (ship.is_active) {
    Collision collision = terrain.GetCollisionFast(&ship);
    // If the ship ran into anything or went out of scope
    // update.
    if (collision.exists) {
      ship.ProcessCollision(&collision);
      return true;
    } else if (collision.out_of_scope) {
      ship.SetInactive();
      return true;
    } else if (ship.pos.y < screen->screen_bottom)  {
      return true;
    } else if (time_left < 0) {
      #ifdef SPEEDTEST
        return false;
      #else
        return true;
      #endif
    }
  }
  return false;
}

bool Environment::IsOnScreen(int x, int y) {
  return (x >= 0) && (x < GRID_WIDTH) && (y < GRID_HEIGHT) && (y >= 0);
}
