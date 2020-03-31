#include "graphics/load_shader.h"
#include "base/googletest.h"

class OpenGLTest : public ::testing::Test {
 protected:
  OpenGLTest() {}

  virtual ~OpenGLTest() {}

  virtual void SetUp() {
    SDL_Init(SDL_INIT_EVERYTHING);
    uint32_t window_flags = SDL_WINDOW_HIDDEN | SDL_WINDOW_OPENGL;
    window = SDL_CreateWindow("Test.", 0, 0, 0, 0, window_flags);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK,
                        SDL_GL_CONTEXT_PROFILE_CORE);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
    gl_context = SDL_GL_CreateContext(window);
    if (!gl_context) {
      LOG(FATAL) << "Couldn't create OpenGL context, error: " << SDL_GetError();
    }
    if (!gladLoadGL()) {
      LOG(FATAL) << "Something went wrong.";
    }
  }

  virtual void TearDown() {
    SDL_GL_DeleteContext(gl_context);
    SDL_DestroyWindow(window);
    SDL_Quit();
  }

  SDL_Window* window;
  SDL_GLContext gl_context;
};

TEST_F(OpenGLTest, Smoke) {
  GLint major = -1;
  GLint minor = -1;
  glGetIntegerv(GL_MAJOR_VERSION, &major);
  glGetIntegerv(GL_MINOR_VERSION, &minor);
  EXPECT_EQ(major, 4);
  EXPECT_EQ(minor, 3);
}

TEST_F(OpenGLTest, LoadShader) {
  LoadShader("graphics/testdata/test_shader.vert", GL_VERTEX_SHADER);
  LoadShader("graphics/testdata/test_shader.frag", GL_FRAGMENT_SHADER);
  LoadShader("graphics/testdata/draw_particles.cs", GL_COMPUTE_SHADER);
}

GTEST_MAIN();
