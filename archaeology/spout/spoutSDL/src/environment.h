#ifndef ENVIRONMENT_H_
#define ENVIRONMENT_H_

#include "constants.h"
#include "motion_constants.h"
#include "types.h"
#include "controller_input.h"

#include "terrain.h"
#include "ship.h"
#include "particle.h"
#include "emitter.h"
#include "vec.h"
#include "collision.h"

class Environment {
  public:
    Screen<counter_t>* screen;
    Screen<counter_t>* particle_screen;
    Ship ship = Ship(Vec_make(GRID_WIDTH/2 + .5, GRID_HEIGHT/2 + .5), M_PI/2);
    int score = 0;
    Terrain terrain;
    RingBuffer<Particle>* particles = &ship.emitter.particles;
    int current_level = -1;
    double time_left = -1;

    Environment(Screen<counter_t>* screen, Screen<counter_t>* particle_screen);
    bool IsOnScreen(int x, int y);
    void CheckCollisions(double time);
    bool IsGameOver();
    void Reset();
    void HandleInput(ControllerInput* input);
    bool Update(double interval);
    Collision HasCollision(Vec p1, Vec p2);
};

#endif
