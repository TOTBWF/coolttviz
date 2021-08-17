#version 330 core

out vec4 color;

void main() {
  vec2 position = (gl_FragCoord.xy / vec2(1024, 768));
  vec4 top = vec4(47.0, 38.0, 183.0, 255.0) / 255.0;
  vec4 bottom = vec4(243.0, 71.0, 176.0, 255.0) / 255.0;

  color = vec4(mix(bottom, top, position.y));
}
