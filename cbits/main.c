#include "GL/gl3w.h"
#include "GL/glcorearb.h"
#include "SDL_events.h"
#include "SDL_keycode.h"
#include <stdlib.h>
#include <string.h>
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include "cimgui.h"
#include "cimgui_extras.h"
#include "cimgui_impl.h"
#include <stdio.h>
#include <stdint.h>
#include <math.h>

#include <cglm/cglm.h>

#define SDL_MAIN_HANDLED
#include <SDL.h>

#ifdef IMGUI_HAS_IMSTR
#define igBegin igBegin_Str
#define igSliderFloat igSliderFloat_Str
#define igCheckbox igCheckbox_Str
#define igColorEdit3 igColorEdit3_Str
#define igButton igButton_Str
#endif

#include "shader.h"

SDL_Window *window = NULL;

/*!
 * Insert a single zero bit into [bits] at position [ix], shifting
 * over all bits to the left of the index.
 */
inline uint32_t insert_bit(uint32_t bits, int ix) {
  uint32_t upper_mask = UINT32_MAX << (ix + 1);
  uint32_t upper = upper_mask & (bits << 1);
  uint32_t lower_mask = (1 << ix) - 1;
  uint32_t lower = lower_mask & bits;
  return upper | lower;
}

inline uint32_t power_of_2(int p) {
  return (1 << p);
}

/*!
 * Project down an n-dimensional vector into a 3-dimensional one.
 */
void project(float *v, int n, float *out) {
  float view_angle = glm_rad(45.0f);
  float t = tan(view_angle / 2.0f);

  float tmp[n];
  memcpy(tmp, v, sizeof(float) * n);

  for(int k = n; k > 3; k--) {
    float proj = tmp[k - 1] + 3.0f;
    for (int i = 0; i < k - 1; i++) {
      tmp[i] = (t * tmp[i]) / proj;
    }
  }

  out[0] = tmp[0];
  out[1] = tmp[1];
  out[2] = tmp[2];
}

// The general plan here is to build up a bunch of lines.
float *hypercube(int n, float size) {
  float e0[n];
  float e1[n];

  // Each line consists of 2 vec3 endpoints, and there are 2^(n - 1)*n lines.
  float *points = malloc(2 * sizeof(float) * 3 * power_of_2(n - 1) * n);
  int point_offset = 0;

  // We start by selecting what dimensions we will be drawing lines along.
  for (int line_dim = 0; line_dim < n; line_dim++) {
    // Next, we select where the line will be positioned.
    for(uint32_t pos = 0; pos < power_of_2(n - 1); pos++) {
      uint32_t point = insert_bit(pos, line_dim);

      // Compute the n-dimensional endpoint vectors, then
      // project them down into their 3d representations.
      for (int i = 0; i < n; i++) {
        if (i == line_dim) {
          e0[i] = -size;
          e1[i] = size;
        } else {
          float c = (1 << i) & point ? size : -size;
          e0[i] = e1[i] = c;
        }
      }

      project(e0, n, points + point_offset);
      point_offset += 3;
      project(e1, n, points + point_offset);
      point_offset += 3;
    }
  }

  return points;
}
GLuint vertex_array;
GLuint vertex_buffer;
GLuint program;

mat4 model;
mat4 view;
mat4 projection;

void init() {
  glGenVertexArrays(1, &vertex_array);
  glGenBuffers(1, &vertex_buffer);

  glBindVertexArray(vertex_array);

  int n = 4;
  float *vertices = hypercube(n, 1.0f);
  int num_vertices = (2 * power_of_2(n - 1) * n);

  glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
  glBufferData(GL_ARRAY_BUFFER, 3 * sizeof(float) * num_vertices, vertices, GL_STATIC_DRAW);

  glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);

  glEnableVertexAttribArray(0);

  // Unbind the VBO/VAO so any further calls do not modify them.
  glBindBuffer(GL_ARRAY_BUFFER, 0);
  glBindVertexArray(0);

  glm_mat4_identity(model);
  glm_mat4_identity(view);
  glm_mat4_identity(projection);

  glm_perspective(glm_rad(45.0f), 800.0f / 600.0f, 0.1f, 100.0f, projection);
  glm_translate(view, (vec3) {0.0f, 0.0f, -1.0f});

  program = load_shader("vertex.glsl", "fragment.glsl");

}

void render() {
  glClearColor(0,0,0,0);
  glClear(GL_COLOR_BUFFER_BIT);

  glUseProgram(program);

  int model_loc = glGetUniformLocation(program, "model");
  int view_loc = glGetUniformLocation(program, "view");
  int projection_loc = glGetUniformLocation(program, "projection");

  glm_rotate(model, glm_rad(1.0f), (vec3) {0.0f, 1.0f, 0.0f});

  glUniformMatrix4fv(model_loc, 1, GL_FALSE, (float*) model);
  glUniformMatrix4fv(view_loc, 1, GL_FALSE, (float*) view);
  glUniformMatrix4fv(projection_loc, 1, GL_FALSE, (float*) projection);

  int n = 4;
  int num_vertices = (2 * power_of_2(n - 1) * n);

  glBindVertexArray(vertex_array);
  glDrawArrays(GL_LINES, 0, num_vertices);
}

int main(int argc, char* argv[]) {

  if (SDL_Init(SDL_INIT_VIDEO) < 0) {
    SDL_Log("failed to init: %s", SDL_GetError());
    return -1;
  }

  // Decide GL+GLSL versions
#if __APPLE__
    // GL 3.2 Core + GLSL 150
    const char* glsl_version = "#version 150";
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_FLAGS, SDL_GL_CONTEXT_FORWARD_COMPATIBLE_FLAG); // Always required on Mac
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 2);
#else
    // GL 3.0 + GLSL 130
    const char* glsl_version = "#version 130";
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_FLAGS, 0);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 0);
#endif

  // and prepare OpenGL stuff
  SDL_SetHint(SDL_HINT_RENDER_DRIVER, "opengl");
  SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 24);
  SDL_GL_SetAttribute(SDL_GL_STENCIL_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
  SDL_DisplayMode current;
  SDL_GetCurrentDisplayMode(0, &current);

  window = SDL_CreateWindow(
      "Hello", 0, 0, 1024, 768,
      SDL_WINDOW_SHOWN | SDL_WINDOW_OPENGL | SDL_WINDOW_RESIZABLE
      );
  if (window == NULL) {
    SDL_Log("Failed to create window: %s", SDL_GetError());
    return -1;
  }

  SDL_GLContext gl_context = SDL_GL_CreateContext(window);
  SDL_GL_SetSwapInterval(1);  // enable vsync

  bool err = Do_gl3wInit() != 0;
  if (err) {
    SDL_Log("Failed to initialize OpenGL loader for cimgui_sdl!");
    return 1;
  }

  igCreateContext(NULL);

  //set docking
  ImGuiIO* ioptr = igGetIO();
  ioptr->ConfigFlags |= ImGuiConfigFlags_NavEnableKeyboard;       // Enable Keyboard Controls
  ImGui_ImplSDL2_InitForOpenGL(window, gl_context);
  ImGui_ImplOpenGL3_Init(glsl_version);

  igStyleColorsDark(NULL);

  init();

  int dim = 2;
  bool quit = false;
  while (!quit)
  {
    SDL_Event e;

    // we need to call SDL_PollEvent to let window rendered, otherwise
    // no window will be shown
    while (SDL_PollEvent(&e) != 0)
    {
      ImGui_ImplSDL2_ProcessEvent(&e);
      if (e.type == SDL_QUIT)
        quit = true;
      if (e.type == SDL_WINDOWEVENT && e.window.event == SDL_WINDOWEVENT_CLOSE && e.window.windowID == SDL_GetWindowID(window))
        quit = true;
      if (e.type == SDL_KEYUP && e.key.keysym.sym == SDLK_ESCAPE)
        quit = true;
    }
    ImGui_ImplOpenGL3_NewFrame();
    ImGui_ImplSDL2_NewFrame(window);
    igNewFrame();
    char *msg;
    int *dims;

    {
      igBegin("Hello, world!", NULL, 0);
      igSliderInt("Dimension", &dim, 2, 10, "%d", 0);

      igText("Dimensions: %s", msg);
      igText("Application average %.3f ms/frame (%.1f FPS)", 1000.0f / igGetIO()->Framerate, igGetIO()->Framerate);
      igEnd();
    }

    igRender();
    /* glViewport(0, 0, (int)ioptr->DisplaySize.x, (int)ioptr->DisplaySize.y); */
    render();

    ImGui_ImplOpenGL3_RenderDrawData(igGetDrawData());
    SDL_GL_MakeCurrent(window, gl_context);
    SDL_GL_SwapWindow(window);

    free(msg);
    free(dims);
  }

  ImGui_ImplOpenGL3_Shutdown();
  ImGui_ImplSDL2_Shutdown();
  igDestroyContext(NULL);

  SDL_GL_DeleteContext(gl_context);
  if (window != NULL)
  {
    SDL_DestroyWindow(window);
    window = NULL;
  }
  SDL_Quit();

  return 0;
}
