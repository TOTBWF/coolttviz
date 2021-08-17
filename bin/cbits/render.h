typedef struct label_ {
  char *contents;
  float *location;
} label;

int render(int dim, int num_labels, label *labels);
