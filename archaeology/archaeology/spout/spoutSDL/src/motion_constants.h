#ifndef MOTION_CONSTANTS_H_
#define MOTION_CONSTANTS_H_
#include "vec.h"

static const int SHIP_ROTATION_SPEED = 15;

#ifndef SPEEDTEST
static const int SHIP_ACCELERATION = 200;
static const Vec GRAVITY = { 0, -SHIP_ACCELERATION/5.0};
static const Vec PARTICLE_GRAVITY = Vec_multiply(GRAVITY, 10);
#else
static const int SHIP_ACCELERATION = 0;
static const Vec GRAVITY = { 0, 0};
static const Vec PARTICLE_GRAVITY = Vec_multiply(GRAVITY, 10);
#endif

static const double COLLISION_ELASTICITY = 0.4;

// Emissions
static const int SHIP_EMISSION_RATE = 100000;
static const double SHIP_EMISSION_SPREAD = M_PI/4;
static const int SHIP_TAIL_LENGTH = 20;
static const double SHIP_EMISSION_MIN_SPEED = 100.0;
static const double SHIP_EMISSION_MAX_SPEED = 500.0;
static const double PARTICLE_LIFE = 2.0;

static const double SPEED_DAMAGE = .05;

static const double FADE_RATE = 500;

static const int INITIAL_TIME = 15; // In seconds
static const int INCREMENTAL_TIME = 15; // In seconds

static const int AVG_SPEED = (SHIP_EMISSION_MAX_SPEED + SHIP_EMISSION_MIN_SPEED)/2;
static const int PARTICLE_PER_SPOT = SHIP_EMISSION_RATE/((AVG_SPEED*AVG_SPEED)/(2.0/SHIP_EMISSION_SPREAD));
// static const int PARTICLE_PER_SPOT = 1;
static const double HEAT_APPEARANCE = .2;

#endif
