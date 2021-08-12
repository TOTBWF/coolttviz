#include "GL/gl3w.h"
#include "GL/glcorearb.h"
#include "SDL_events.h"
#include "SDL_keycode.h"
#include <stdlib.h>
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include "cimgui.h"
#include "cimgui_extras.h"
#include "cimgui_impl.h"
#include <stdio.h>
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

int choose(int n, int k) {
  int r = 1;
  for(int i = 1; i <= k; i++, n--) {
    r = r/i * n + r%i * n/i;
  }
  return r;
}

int *faces(int n) {
  int *cs = malloc(sizeof(int) * choose(n, 2) * 2);
  int ix = 0;
  for (int i = 0; i <= n - 2; i++) {
    for (int j = i + 1; j <= n - 1; j++) {
      if (ix >= choose(n, 2) * 2) {
        printf("Choose: %d\n", choose(n, 2) * 2);
        exit(1);
      }
      cs[ix] = i;
      cs[ix + 1] = j;
      ix += 2;
    }
  }
  return cs;
}

// So what I want to do here is just feed these faces into the vertex shader?
// Yeah, these are the 2 dimensions we vary to form the face. We then need to iterate
// over all of the possible faces that can have those faces vary. For a 3-cube this is
// easy (there is just 1)

GLuint vertex_array;
GLuint vertex_buffer;
GLuint element_buffer;
GLuint program;

mat4 model;
mat4 view;
mat4 projection;

float vertices[] = {
    0.5f,  0.5f,  0.0f, // top right
    0.5f,  -0.5f, 0.0f, // bottom right
    -0.5f, -0.5f, 0.0f, // bottom left
    -0.5f, 0.5f,  0.0f  // top left
};
unsigned int indices[] = {
    // note that we start from 0!
    0, 1, 3, // first Triangle
    1, 2, 3  // second Triangle
};

void init() {
  glGenVertexArrays(1, &vertex_array);
  glGenBuffers(1, &vertex_buffer);
  glGenBuffers(1, &element_buffer);

  glBindVertexArray(vertex_array);

  glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
  glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

  glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element_buffer);
  glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(indices), indices, GL_STATIC_DRAW);

  glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);

  glEnableVertexAttribArray(0);
  glBindBuffer(GL_ARRAY_BUFFER, 0);
  glBindVertexArray(0);

  glm_mat4_identity(model);
  glm_mat4_identity(view);
  glm_mat4_identity(projection);

  glm_perspective(glm_rad(45.0f), 800.0f / 600.0f, 0.1f, 100.0f, projection);
  glm_translate(view, (vec3) {0.0f, 0.0f, -3.0f});

  program = load_shader("vertex.glsl", "fragment.glsl");

  glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
}


void render() {
  glClearColor(0,0,0,0);
  glClear(GL_COLOR_BUFFER_BIT);

  glUseProgram(program);

  int model_loc = glGetUniformLocation(program, "model");
  int view_loc = glGetUniformLocation(program, "view");
  int projection_loc = glGetUniformLocation(program, "projection");

  glUniformMatrix4fv(model_loc, 1, GL_FALSE, (float*) model);
  glUniformMatrix4fv(view_loc, 1, GL_FALSE, (float*) view);
  glUniformMatrix4fv(projection_loc, 1, GL_FALSE, (float*) projection);

  glBindVertexArray(vertex_array);
  glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);
  /* glDrawArrays(GL_TRIANGLES, 0, 3); */
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

      dims = faces(dim);
      asprintf(&msg, "Dim: %d", dim);

      msg[0] = '\0';
      for(int i = 0; i < choose(dim, 2); i++) {
        char *dim_str;
        asprintf(&dim_str, "{%d, %d}", dims[2*i], dims[2*i+1]);
        strcat(msg, dim_str);
      }

      igText("Dimensions: %s", msg);
      igText("Application average %.3f ms/frame (%.1f FPS)", 1000.0f / igGetIO()->Framerate, igGetIO()->Framerate);
      igEnd();
    }

    igRender();
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
