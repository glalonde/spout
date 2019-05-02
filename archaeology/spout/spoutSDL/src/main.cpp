//The headers
#include "SDL2/SDL.h"
//#include <SDL2/SDL_mixer.h>
#include "GL/glew.h"

#include <SDL2/SDL_opengl.h>
#include <OpenGL/gl.h>
#include <OpenGL/glu.h>
#include <pthread.h>
#include <time.h>   

#include "timer.h"
#include "constants.h"
#include "spout_game.h"
#include "types.h"
#include "screen.h"
#include "assert.h"
#include "gl_renderer.h"


//#include <xmmintrin.h>
//#include <pmmintrin.h>

bool initGL() {
  //Initialize Projection Matrix
  glMatrixMode( GL_PROJECTION );
  glLoadIdentity();
  
  //Initialize Modelview Matrix
  glMatrixMode( GL_MODELVIEW );
  glLoadIdentity();

  //Initialize clear color
  glClearColor( 0.f, 0.f, 0.f, 1.f );

  //Check for error
  GLenum error = glGetError();
  if( error != GL_NO_ERROR ) {
    printf( "Error initializing OpenGL! %s\n", gluErrorString( error ) );
    return false;
  }

  return true;
}

bool Init(SDL_Window** win, SDL_GLContext* gl_context) {
  //Initialization flag
  bool success = true;

  //Use OpenGL 2.1
  SDL_GL_SetAttribute( SDL_GL_CONTEXT_MAJOR_VERSION, 3 );
  SDL_GL_SetAttribute( SDL_GL_CONTEXT_MINOR_VERSION, 1 );
  SDL_GL_SetAttribute( SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE );

  //Create window
  *win = SDL_CreateWindow( "SDL Tutorial", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, SCREEN_WIDTH, SCREEN_HEIGHT, WINDOW_MODE | SDL_WINDOW_OPENGL);
  if(*win == NULL) {
    printf( "Window could not be created! SDL Error: %s\n", SDL_GetError() );
    success = false;
  } else {
    //Create context
    *gl_context = SDL_GL_CreateContext(*win);
    if(*gl_context == NULL) {
      printf( "OpenGL context could not be created! SDL Error: %s\n", SDL_GetError() );
      success = false;
    } else {
      //Initialize GLEW
      glewExperimental = GL_TRUE; 
      GLenum glewError = glewInit();
      if( glewError != GLEW_OK ) {
        printf( "Error initializing GLEW! %s\n", glewGetErrorString( glewError ) );
      }

      //Use Vsync
      if( SDL_GL_SetSwapInterval( 1 ) < 0 ) {
        printf( "Warning: Unable to set VSync! SDL Error: %s\n", SDL_GetError() );
      }

      //Initialize OpenGL
      if( !initGL() ) {
        printf( "Unable to initialize OpenGL!\n" );
        success = false;
      }
    }
  }

  SDL_ShowCursor(0);

  // Mix_OpenAudio( MIX_DEFAULT_FREQUENCY, MIX_DEFAULT_FORMAT, 2, 4096);
  // *music = Mix_LoadMUS(MUSIC_PATH);

  return success;
}

// Free the graphics structs
void CleanUp(SDL_Window* win, SDL_GLContext gl_context) {
  SDL_DestroyWindow(win);
  SDL_GL_DeleteContext(gl_context);
  SDL_Quit();

  // Mix_FreeMusic(music);
  // Mix_CloseAudio();
}


bool UpdateInput(SDL_Event* event, ControllerInput* input, double interval) {
  input->time = interval;
  while(SDL_PollEvent(event)) {
    switch (event->type) {
      case SDL_QUIT: return true;
      case SDL_KEYDOWN:
      case SDL_KEYUP:
        bool set_val = (event->type == SDL_KEYDOWN);
        switch (event->key.keysym.sym) {
          case SDLK_q: return true;
          case SDLK_UP: input->up = set_val; break;
          case SDLK_DOWN: input->down = set_val; break;
          case SDLK_LEFT: input->left = set_val; break;
          case SDLK_RIGHT: input->right = set_val; break;
          case SDLK_r: input->reset = set_val; break;
          case SDLK_p: input->pause = set_val; break;
        }
    }
  }
  return false;
}

struct ArgStruct {
  bool* is_playing;
  bool* frame_ready;
  pthread_mutex_t* sync_mutex;
  pthread_cond_t* consumer_done;
  pthread_cond_t* producer_done;

  // For update loop
  SpoutGame* game;
};

void* UpdateLoop(void* argptr) {
  ArgStruct* args = (ArgStruct*) argptr;
  // Loop timer
  Timer delta_timer;

  // Input structs
  SDL_Event event;
  ControllerInput input;
  delta_timer.start();

  double interval;

  while(*args->is_playing) {
    pthread_mutex_lock(args->sync_mutex);
    // Wait while the frame hasn't been consumed...
    while (*(args->frame_ready)) {
      pthread_cond_wait(args->consumer_done, args->sync_mutex);
    }

    interval = delta_timer.get_ticks()/((double)TICKS_PER_SECOND);
    // Reset timer for next iteration
    delta_timer.start();
    *(args->is_playing) = !UpdateInput(&event, &input, interval);

    // Update game state
    args->game->HandleInput(&input);
    args->game->Update(interval);
    args->game->Draw();

    *(args->frame_ready) = true;
    pthread_cond_signal(args->producer_done);
    pthread_mutex_unlock(args->sync_mutex);
  }
  return NULL;
}

void GameLoop(counter_t pixels1[GRID_HEIGHT][GRID_WIDTH], counter_t pixels2[GRID_HEIGHT][GRID_WIDTH], SDL_Window* win) {
  // The main canvas. Has two buffers.
  Screen<counter_t> screen(pixels1);
  Screen<counter_t> particle_screen(pixels2);

  pthread_t update_thread;

  // Game object
  SpoutGame* game = new SpoutGame(&screen, &particle_screen);

  // Start the music
  // Mix_PlayMusic( music, -1);

  bool is_playing = true;

  // Synchronization primitives
  pthread_mutex_t sync_mutex = PTHREAD_MUTEX_INITIALIZER;
  pthread_cond_t producer_done = PTHREAD_COND_INITIALIZER;
  pthread_cond_t consumer_done = PTHREAD_COND_INITIALIZER;
  bool frame_ready = false;

  ArgStruct thread_args;
  thread_args.is_playing = &is_playing;
  thread_args.frame_ready = &frame_ready;
  thread_args.sync_mutex = &sync_mutex;
  thread_args.producer_done = &producer_done;
  thread_args.consumer_done = &consumer_done;
  thread_args.game = game;

  // start threads
  pthread_create(&update_thread, NULL, &UpdateLoop, (void*)&thread_args);
  RenderLoop(&screen,
             &particle_screen,
             &is_playing,
             &frame_ready,
             &sync_mutex,
             &producer_done,
             &consumer_done,
             win);

  // join threads
  pthread_join(update_thread, NULL);

  delete game;
}

int main(int argc, char* args[]) {
  SDL_Init( SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_EVENTS);
  SDL_Window* win = NULL;
  SDL_GLContext gl_context = NULL;
  // Mix_Music* music = NULL;
  static counter_t pixels1[GRID_HEIGHT][GRID_WIDTH];
  static counter_t pixels2[GRID_HEIGHT][GRID_WIDTH];

  // Flush denormals to zero, just in case.
  //_MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON);
  //_MM_SET_DENORMALS_ZERO_MODE(_MM_DENORMALS_ZERO_ON);
  
  Init(&win, &gl_context);


  // Start loop
  GameLoop(pixels1, pixels2, win);
  
  CleanUp(win, gl_context);
  return 0;
}
