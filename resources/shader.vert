#version 140

in vec3 position;
in vec4 color;

out vec4 v_color;

uniform mat4 model;
uniform mat4 view_projection;

void main() {
  gl_Position = view_projection * model * vec4(position, 1.0);
  v_color = color;
}
