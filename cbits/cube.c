#include <cglm/cglm.h>

#include <string.h>
#include <stdint.h>
#include <math.h>

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

inline uint32_t power_of_2(int p) {
  return (1 << p);
}

int hypercube_vertices(int n) {
  return power_of_2(n) * n;
}

float *hypercube(int n, float size) {
  float e0[n];
  float e1[n];

  // Each line consists of 2 vec3 endpoints, and there are 2^(n - 1)*n lines.
  float *points = malloc(3 * sizeof(float) * hypercube_vertices(n) * n);
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

