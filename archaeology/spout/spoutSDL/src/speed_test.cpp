#include "SDL2/SDL.h"
#include "timer.h"
#include "constants.h"
#include "spout_game.h"
#include "types.h"
#include "screen.h"
#include "assert.h"
#include "ktiming.h"
#include "gl_renderer.h"


static const int NUM_FRAMES = 10000;


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

bool Init(SDL_Window** win, SDL_GLContext* gl_context, Mix_Music** music) {
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

  Mix_OpenAudio( MIX_DEFAULT_FREQUENCY, MIX_DEFAULT_FORMAT, 2, 4096);
  *music = Mix_LoadMUS(MUSIC_PATH);

  return success;
}

// Free the graphics structs
void CleanUp(SDL_Window* win, SDL_GLContext gl_context, Mix_Music* music) {
  SDL_DestroyWindow(win);
  SDL_GL_DeleteContext(gl_context);
  SDL_Quit();

  Mix_FreeMusic(music);
  Mix_CloseAudio();
}

bool UpdateInput(ControllerInput* input, double interval) {
  input->time = interval;
  input->up = true;
  input->right = true;
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
  int frames = 0;

  // Input structs
  ControllerInput input;
  double interval = 1.0/60.0;

  while(*args->is_playing) {
    pthread_mutex_lock(args->sync_mutex);
    // Wait while the frame hasn't been consumed...
    while (*(args->frame_ready)) {
      pthread_cond_wait(args->consumer_done, args->sync_mutex);
    }

    // Reset timer for next iteration
    *(args->is_playing) = frames < NUM_FRAMES;
    UpdateInput(&input, interval);

    // Update game state
    args->game->HandleInput(&input);
    args->game->Update(interval);
    args->game->Draw();

    *(args->frame_ready) = true;
    pthread_cond_signal(args->producer_done);
    pthread_mutex_unlock(args->sync_mutex);
    frames++;
  }
  return NULL;
}

void GameLoop(counter_t pixels1[GRID_HEIGHT][GRID_WIDTH], counter_t pixels2[GRID_HEIGHT][GRID_WIDTH], SDL_Window* win, Mix_Music* music) {
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
  uint64_t start_time = ktiming_getmark();
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

  uint64_t end_time = ktiming_getmark();
  
  double time = ktiming_diff_sec(&start_time, &end_time);
  printf("Elapsed Time: %f\n", time);
  printf("FPS: %f\n", NUM_FRAMES/time);

  delete game;
}


int main(int argc, char* args[]) {
  SDL_Init( SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_EVENTS);
  SDL_Window* win = NULL;
  SDL_GLContext gl_context = NULL;
  Mix_Music* music = NULL;
  static counter_t pixels1[GRID_HEIGHT][GRID_WIDTH];
  static counter_t pixels2[GRID_HEIGHT][GRID_WIDTH];

  Init(&win, &gl_context, &music);

  // Start loop
  GameLoop(pixels1, pixels2, win, music);
  
  CleanUp(win, gl_context, music);
  return 0;
}
