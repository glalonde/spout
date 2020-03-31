#include <SDL2/SDL.h>
#include <SDL2/SDL_mixer.h>
 
#define MUS_PATH "sexy3.xm"
 
// Our music file
Mix_Music *music = NULL;
 
 
int main(int argc, char* argv[]){
 
  // Initialize SDL.
  if (SDL_Init(SDL_INIT_AUDIO) < 0)
    return -1;
      
  //Initialize SDL_mixer 
  if( Mix_OpenAudio( MIX_DEFAULT_FREQUENCY, MIX_DEFAULT_FORMAT, 2, 4096) == -1 ) 
    return -1; 
  
  // Load our music
  music = Mix_LoadMUS(MUS_PATH);
  if (music == NULL)
    return -1;
  
  if ( Mix_PlayMusic( music, -1) == -1 )
    return -1;
    
  while ( Mix_PlayingMusic() ) {
    SDL_Delay(50);
  };
  
  // clean up our resources
  Mix_FreeMusic(music);
  
  // quit SDL_mixer
  Mix_CloseAudio();
  
  return 0;
}
