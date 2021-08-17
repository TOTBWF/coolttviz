#if __APPLE__
#define GL_SILENCE_DEPRECATION
#include <OpenGL/gl.h>
#include <OpenGL/glu.h>
#endif

GLuint load_static_shader(const char *vertex_shader_code, const char *frag_shader_code);
GLuint load_shader(const char *vertex_shader_path, const char *frag_shader_path);
