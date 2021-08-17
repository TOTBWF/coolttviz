#include "GL/gl3w.h"
#include "GL/glcorearb.h"
#include "SDL_events.h"
#include "SDL_keycode.h"
#include "cglm/affine.h"
#include "cglm/cam.h"
#include "cglm/mat4.h"
#include "cglm/project.h"
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
#include "cube.h"

SDL_Window *window = NULL;

typedef struct scene_ {
  int window_width;
  int window_height;

  // Camera Controls.
  float azimuth;
  float polar;
  float radius;

  GLuint cube_vao;
  GLuint cube_vbo;
  float *cube_vertices;
  int dim;

  mat4 model;
  mat4 view;
  mat4 projection;

  GLuint shader;
} scene;

void init(int n, scene* scene) {
  scene->dim = n;

  glm_mat4_identity(scene->model);
  glm_mat4_identity(scene->view);
  glm_mat4_identity(scene->projection);

  glGenVertexArrays(1, &scene->cube_vao);
  glGenBuffers(1, &scene->cube_vbo);

  glBindVertexArray(scene->cube_vao);

  float *cube_vertices = hypercube(n, 1.0f);
  int num_vertices = hypercube_vertices(n);

  glBindBuffer(GL_ARRAY_BUFFER, scene->cube_vbo);
  glBufferData(GL_ARRAY_BUFFER, 3 * sizeof(float) * num_vertices, cube_vertices, GL_DYNAMIC_DRAW);

  glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);

  glEnableVertexAttribArray(0);

  glBindBuffer(GL_ARRAY_BUFFER, 0);
  glBindVertexArray(0);

  scene->shader = load_shader("vertex.glsl", "fragment.glsl");
}

void render_label(scene* scene, char *label, float *label_pos) {
  vec4 label_pos_proj;

  project(label_pos, scene->dim, label_pos_proj);
  // We need to set the 'w' component to 0 here to make the conversion
  // into Normalized Device Coordinates work.
  label_pos_proj[3] = 1.0f;

  glm_mat4_mulv(scene->model, label_pos_proj, label_pos_proj);
  glm_mat4_mulv(scene->view, label_pos_proj, label_pos_proj);
  glm_mat4_mulv(scene->projection, label_pos_proj, label_pos_proj);

  float x_ndc = label_pos_proj[0] / label_pos_proj[3];
  float y_ndc = label_pos_proj[1] / label_pos_proj[3];

  ImVec2 window_pos;
  window_pos.x = ((1.0f + x_ndc) / 2) * scene->window_width;
  window_pos.y = ((1.0f - y_ndc) / 2) * scene->window_height;

  ImVec2 pivot;
  pivot.x = 0.0f;
  pivot.y = 0.0f;

  igSetNextWindowPos(window_pos, ImGuiCond_Always, pivot);
  igBegin(label, NULL, 0);
  igText("Look, more stuff!");
  igEnd();
}

void render_frame(scene *scene) {
  igNewFrame();

  glViewport(0, 0, scene->window_width, scene->window_height);

  glClearColor(0,0,0,0);
  glClear(GL_COLOR_BUFFER_BIT);

  glUseProgram(scene->shader);

  int model_loc = glGetUniformLocation(scene->shader, "model");
  int view_loc = glGetUniformLocation(scene->shader, "view");
  int projection_loc = glGetUniformLocation(scene->shader, "projection");

  float aspect = ((float) scene->window_width) / ((float) scene->window_height);
  vec3 eye = {
    scene->radius * cos(scene->polar) * cos(scene->azimuth),
    scene->radius * sin(scene->polar),
    scene->radius * cos(scene->polar) * sin(scene->azimuth)
  };
  vec3 origin = { 0.0f, 0.0f, 0.0f };
  vec3 up = { 0.0f, 1.0f, 0.0f };

  glm_lookat(eye, origin, up, scene->view);
  glm_perspective(glm_rad(45.0f), aspect, 0.1f, 100.0f, scene->projection);

  glUniformMatrix4fv(model_loc, 1, GL_FALSE, (float*) scene->model);
  glUniformMatrix4fv(view_loc, 1, GL_FALSE, (float*) scene->view);
  glUniformMatrix4fv(projection_loc, 1, GL_FALSE, (float*) scene->projection);

  int num_vertices = hypercube_vertices(scene->dim);

  glBindVertexArray(scene->cube_vao);
  glDrawArrays(GL_LINES, 0, num_vertices);

  igRender();
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

  int dim = 2;
  scene scene;
  scene.radius = 4.0f;
  init(dim, &scene);

  bool quit = false;
  bool mouse_down = false;
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
      // Keyboard
      if (!ioptr->WantCaptureKeyboard) {
        if (e.type == SDL_KEYUP && e.key.keysym.sym == SDLK_ESCAPE)
          quit = true;
      }
      // Mouse
      if (!ioptr->WantCaptureMouse) {
        if (e.type == SDL_MOUSEWHEEL && e.wheel.y > 0)
          scene.radius += 0.1;
        if (e.type == SDL_MOUSEWHEEL && e.wheel.y < 0)
          scene.radius -= 0.1;
        if (e.type == SDL_MOUSEBUTTONDOWN)
          mouse_down = true;
        if (e.type == SDL_MOUSEBUTTONUP)
          mouse_down = false;
        if (e.type == SDL_MOUSEMOTION && mouse_down) {
          // FIXME: If the polar angle goes over pi/2 radians, then things flip.
          scene.azimuth += e.motion.xrel / 100.0f;
          scene.polar += e.motion.yrel / 100.0f;
        }
      }
    }
    ImGui_ImplOpenGL3_NewFrame();
    ImGui_ImplSDL2_NewFrame(window);

    scene.window_width = ioptr->DisplaySize.x;
    scene.window_height = ioptr->DisplaySize.y;

    render_frame(&scene);

    ImGui_ImplOpenGL3_RenderDrawData(igGetDrawData());
    SDL_GL_MakeCurrent(window, gl_context);
    SDL_GL_SwapWindow(window);
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
