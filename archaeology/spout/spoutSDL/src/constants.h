#ifndef CONSTANTS_H_
#define CONSTANTS_H_

#include "motion_constants.h"
#include "SDL2/SDL.h"



//The screen attributes
#define FULL_SCREEN

static const int SCALE = 4;

#ifdef FULL_SCREEN
static const int WINDOW_MODE = SDL_WINDOW_FULLSCREEN;
static const int SCREEN_WIDTH = 1366;
static const int SCREEN_HEIGHT = 768;
//static const int SCREEN_WIDTH = 2560;
//static const int SCREEN_HEIGHT = 1440;
static const int GRID_WIDTH = SCREEN_WIDTH/SCALE;
static const int GRID_HEIGHT = SCREEN_HEIGHT/SCALE;
#else
static const int WINDOW_MODE = SDL_WINDOW_SHOWN;
static const int GRID_WIDTH = 160;
static const int GRID_HEIGHT = 90;
static const int SCREEN_WIDTH = GRID_WIDTH*SCALE;
static const int SCREEN_HEIGHT = GRID_HEIGHT*SCALE;
#endif

static const int FRAME_RATE = 60;

// Terrain constants
//Dimensions of the levels
static const int LEVEL_WIDTH = GRID_WIDTH; //Width should probably bee the same as the viewing portal
static const int LEVEL_HEIGHT = 2*GRID_HEIGHT;

static const int FIRST_LEVEL_EMPTY_HEIGHT = LEVEL_HEIGHT/3;

static const float TICKS_PER_SECOND = 1000.f;

static const double EPSILON = .001;

const int FONT_WIDTH = 8;
const int FONT_HEIGHT = 8;

extern const char MUSIC_PATH[50];

#endif
