#include "SDL2/SDL.h"
#include "GL/glew.h"
#include <SDL2/SDL_opengl.h>
#include <GL/gl.h>
#include <GL/glu.h>
#include <pthread.h>
#include "timer.h"
#include "constants.h"
#include "spout_game.h"
#include "types.h"
#include "screen.h"
#include "shaderloader.h"
#include "palette.h"


void InitTex(GLuint* tex, int tex_unit) {
  glGenTextures(1, tex);
  glActiveTexture(GL_TEXTURE0 + tex_unit);
  glBindTexture(GL_TEXTURE_2D, *tex);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE); 
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE); 
}


void* RenderLoop(Screen<counter_t>* screen,
                 Screen<counter_t>* particle_screen,
                 bool* is_playing,
                 bool* frame_ready,
                 pthread_mutex_t* sync_mutex,
                 pthread_cond_t* producer_done,
                 pthread_cond_t* consumer_done,
                 SDL_Window* win) {

  glDisable(GL_DEPTH_TEST);
  glDisable(GL_BLEND);

  // Set up the color mapping for the particles
  pixel_t particle_colors[255];
  ParticlePalette::GetPalette(particle_colors);

  // Set up the color mapping for the terrain
  pixel_t terrain_colors[255];
  TerrainPalette::GetPalette(terrain_colors);

  // Compile shaders
  GLuint coloring_program = LoadShader("shaders/basic.vert", "shaders/next.frag");
  GLuint glow_pass = LoadShader("shaders/basic.vert", "shaders/glow.frag");
  GLuint scale_pass = LoadShader("shaders/basic.vert", "shaders/basic.frag");

  // No idea why the fuck this is necessary
  GLuint vao;
  glGenVertexArrays(1, &vao);
  glBindVertexArray(vao);

  // Create vertex buffers for the main canvas
  GLuint vbo;
  glGenBuffers(1, &vbo);
  GLfloat vertices[] = {
      // X,  Y, Z, tex x, tex y
      -1.0f, 1.0f,  0.0f, 1.0f,  0.0f, 1.0f, // Top-left
      1.0f, 1.0f,   0.0f, 1.0f,  1.0f, 1.0f, // Top-right
      1.0f, -1.0f,  0.0f, 1.0f,  1.0f, 0.0f, // Bottom-right
      -1.0f, -1.0f, 0.0f, 1.0f,  0.0f, 0.0f  // Bottom-left
  };
  glBindBuffer(GL_ARRAY_BUFFER, vbo);
  glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

  GLuint ebo;
  glGenBuffers(1, &ebo);

  GLuint elements[] = {
      0, 1, 2,
      2, 3, 0
  };

  glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
  glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(elements), elements, GL_STATIC_DRAW);

  // Set up the glow framebuffer
  GLuint fbo;
  glGenFramebuffers(1, &fbo);
  glBindFramebuffer(GL_FRAMEBUFFER, fbo);

  GLuint color_tex;
  InitTex(&color_tex, 3);
  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, GRID_WIDTH, GRID_HEIGHT, 0, GL_RGBA, GL_UNSIGNED_INT_8_8_8_8, NULL);
  glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, color_tex, 0);

  // Second frame buffer
  GLuint fbo2;
  glGenFramebuffers(1, &fbo2);
  glBindFramebuffer(GL_FRAMEBUFFER, fbo2);

  GLuint glow_tex;
  InitTex(&glow_tex, 4);
  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, GRID_WIDTH, GRID_HEIGHT, 0, GL_RGBA, GL_UNSIGNED_INT_8_8_8_8, NULL);
  glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, glow_tex, 0);

  ///////////////////////////

  glUseProgram(coloring_program);
  glBindFragDataLocation(coloring_program, 0, "color_out");

  // Set up the main terrain texture
  glUniform1i(glGetUniformLocation(coloring_program, "terrain_texture"), 0);
  GLuint terrain_tex;
  InitTex(&terrain_tex, 0);

  // Set up the main particle texture
  glUniform1i(glGetUniformLocation(coloring_program, "particle_texture"), 1);
  GLuint particle_tex;
  InitTex(&particle_tex, 1);

  // Set up the main particle color table texture
  glUniform1i(glGetUniformLocation(coloring_program, "particle_palette"), 2);
  GLuint particle_palette = 0;
  InitTex(&particle_palette, 2);

  // Set up the main particle color table texture
  glUniform1i(glGetUniformLocation(coloring_program, "terrain_palette"), 5);
  GLuint terrain_palette = 0;
  InitTex(&terrain_palette, 5);

  // Specify the layout of the vertex data
  GLint pos_attrib = glGetAttribLocation(coloring_program, "pass_position");
  glEnableVertexAttribArray(pos_attrib);
  glVertexAttribPointer(pos_attrib, 4, GL_FLOAT, GL_FALSE, 6 * sizeof(GLfloat), 0);

  // Specify the layout of the vertex data
  GLint tex_attrib = glGetAttribLocation(coloring_program, "pass_texture_coord");
  glEnableVertexAttribArray(tex_attrib);
  glVertexAttribPointer(tex_attrib, 2, GL_FLOAT, GL_FALSE, 6 * sizeof(GLfloat), (GLvoid*)(4*sizeof(GLfloat)));

  // Download the particle palette
  glActiveTexture(GL_TEXTURE2);
  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, 256, 1, 0, GL_RGBA, GL_UNSIGNED_INT_8_8_8_8, (GLvoid*)particle_colors);

  // Download the terrain palette
  glActiveTexture(GL_TEXTURE5);
  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, 256, 1, 0, GL_RGBA, GL_UNSIGNED_INT_8_8_8_8, (GLvoid*)terrain_colors);

  ///////////////////////////

  // Now set up the glow pass
  glUseProgram(glow_pass);

  glUniform1i(glGetUniformLocation(glow_pass, "diffuse_texture"), 3);
  glBindFragDataLocation(glow_pass, 0, "color_out");

  glUniform2f(glGetUniformLocation(glow_pass, "resolution"), GRID_WIDTH, GRID_HEIGHT);

  GLint pos_attrib_2 = glGetAttribLocation(glow_pass, "pass_position");
  glEnableVertexAttribArray(pos_attrib_2);
  glVertexAttribPointer(pos_attrib_2, 2, GL_FLOAT, GL_FALSE, 6 * sizeof(GLfloat), 0);

  GLint tex_attrib_2 = glGetAttribLocation(glow_pass, "pass_texture_coord");
  glEnableVertexAttribArray(tex_attrib_2);
  glVertexAttribPointer(tex_attrib_2, 2, GL_FLOAT, GL_FALSE, 6 * sizeof(GLfloat), (GLvoid*)(4*sizeof(GLfloat)));

  ///////////////////////////

  // Now set up the scale pass
  glUseProgram(scale_pass);
  glUniform1i(glGetUniformLocation(scale_pass, "diffuse_texture"), 4);
  glBindFragDataLocation(scale_pass, 0, "color_out");

  GLint pos_attrib_3 = glGetAttribLocation(scale_pass, "pass_position");
  glEnableVertexAttribArray(pos_attrib_3);
  glVertexAttribPointer(pos_attrib_3, 2, GL_FLOAT, GL_FALSE, 6 * sizeof(GLfloat), 0);

  GLint tex_attrib_3 = glGetAttribLocation(scale_pass, "pass_texture_coord");
  glEnableVertexAttribArray(tex_attrib_3);
  glVertexAttribPointer(tex_attrib_3, 2, GL_FLOAT, GL_FALSE, 6 * sizeof(GLfloat), (GLvoid*)(4*sizeof(GLfloat)));

  ///////////////////////////

  GLenum error = glGetError();
  if( error != GL_NO_ERROR ) {
    printf( "Error initializing OpenGL! %s\n", gluErrorString( error ) );
    return 0;
  }
  GLenum status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
  if (status == GL_FRAMEBUFFER_COMPLETE) {
    printf( "Framebuffer is all good!\n");
  } else {
    printf( "Error initializing framebuffer! Code: %d\n", status);
    return 0;
  }

  glPixelStorei(GL_UNPACK_ALIGNMENT, 1); 
  while(*is_playing) {
    pthread_mutex_lock(sync_mutex);
    // Wait while there is no frame ready
    while (!(*frame_ready)) {
      pthread_cond_wait(producer_done, sync_mutex);
    }

    // Move newest terrain texture to GPU
    glActiveTexture(GL_TEXTURE0);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_R8, GRID_WIDTH, GRID_HEIGHT, 0, GL_RED, GL_UNSIGNED_BYTE, (GLvoid*)screen->cell_buffer);

    // Move newest particle texture to GPU
    glActiveTexture(GL_TEXTURE1);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_R8, GRID_WIDTH, GRID_HEIGHT, 0, GL_RED, GL_UNSIGNED_BYTE, (GLvoid*)particle_screen->cell_buffer);

    // Now release the other thread, since all of the data is on the GPU, we won't access it locally again and concurrent edits are therefore alright.
    *frame_ready = false;
    pthread_cond_signal(consumer_done);
    pthread_mutex_unlock(sync_mutex);

    glUseProgram(coloring_program);
    glBindFramebuffer(GL_FRAMEBUFFER, fbo);
    glViewport(0, 0, GRID_WIDTH, GRID_HEIGHT);
    // Clear existing color buffer
    glClear(GL_COLOR_BUFFER_BIT);
    // Render to framebuffer
    glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);

    // Unbind the framebuffer, so the next pass sends to the back buffer.
    glUseProgram(glow_pass);
    glBindFramebuffer(GL_FRAMEBUFFER, fbo2);
    glClear(GL_COLOR_BUFFER_BIT);
    glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);

    glUseProgram(scale_pass);
    glBindFramebuffer(GL_FRAMEBUFFER, 0);
    glViewport(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
    glClear(GL_COLOR_BUFFER_BIT);
    glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);

    #ifndef SPEEDTEST
    SDL_GL_SwapWindow(win);
    #endif
  }

  // Delete the buffer objects
  glDeleteBuffers(1, &ebo);
  glDeleteBuffers(1, &vbo);
  glDeleteVertexArrays(1, &vao);
  glDeleteFramebuffers(1, &fbo);
  return NULL;
}
