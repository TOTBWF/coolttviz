#if __APPLE__
#define GL_SILENCE_DEPRECATION
#include <OpenGL/gl.h>
#include <OpenGL/glu.h>
#endif

GLuint load_shader(const char *vertex_shader_path, const char *frag_shader_path);
