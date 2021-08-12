
#include "GL/gl3w.h"

#include <stdlib.h>
#include <stdio.h>

#include <sys/stat.h>
#include <sys/types.h>



size_t file_size(FILE *fp) {
  struct stat finfo;
  fstat(fileno(fp), &finfo);
  return (size_t) finfo.st_size;
}

char *read_shader(const char *shader_path) {
  FILE *shader_file = fopen(shader_path, "r");
  size_t shader_size = file_size(shader_file);
  char *shader_code = malloc(shader_size + 1);
  fread(shader_code, 1, shader_size, shader_file);
  shader_code[shader_size] = 0;
  return shader_code;
}

void compile_shader(GLuint shader, char const *shader_code) {
  GLint status;
  GLint info_log_len;

  glShaderSource(shader, 1, &shader_code, NULL);
  glCompileShader(shader);

  glGetShaderiv(shader, GL_COMPILE_STATUS, &status);
  glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &info_log_len);

  if(!status) {
    char *shader_log = malloc(info_log_len);
    glGetShaderInfoLog(shader, info_log_len, NULL, shader_log);
    printf("%s\n", shader_log);
    free(shader_log);
  }
}


GLuint load_shader(const char *vertex_shader_path, const char *frag_shader_path) {
  GLuint vertex_shader = glCreateShader(GL_VERTEX_SHADER);
  GLuint frag_shader = glCreateShader(GL_FRAGMENT_SHADER);


  printf("Compiling Vertex Shader: %s\n", vertex_shader_path);
  char const *vertex_shader_code = read_shader(vertex_shader_path);
  printf("Shader Code: %s\n", vertex_shader_code);
  compile_shader(vertex_shader, vertex_shader_code);

  printf("Compiling Fragment Shader: %s\n", frag_shader_path);
  char const *frag_shader_code = read_shader(frag_shader_path);
  printf("Shader Code: %s\n", frag_shader_code);
  compile_shader(frag_shader, frag_shader_code);

  printf("Linking Shaders...\n");
  GLuint program = glCreateProgram();
  glAttachShader(program, vertex_shader);
  glAttachShader(program, frag_shader);
  glLinkProgram(program);

  GLint status;
  GLint info_log_len;
  glGetProgramiv(program, GL_LINK_STATUS, &status);
  glGetProgramiv(program, GL_INFO_LOG_LENGTH, &info_log_len);
  if(!status) {
    char *program_log = malloc(info_log_len);
    glGetProgramInfoLog(program, info_log_len, NULL, program_log);
    printf("%s\n", program_log);
    free(program_log);
  }

  glDetachShader(program, vertex_shader);
  glDetachShader(program, frag_shader);
  glDeleteShader(vertex_shader);
  glDeleteShader(frag_shader);

  free((char*) vertex_shader_code);
  free((char*) frag_shader_code);

  return program;
}
