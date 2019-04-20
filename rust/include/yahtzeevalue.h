/* From https://youtu.be/zmtHaZG7pPc?t=22m19s */
typedef void yahtzeevalue_t;

struct yahtzeevalue_error {
    char *message;
    int failed;
    int code;
};

void yahtzeevalue_init();

yahtzeevalue_t *yahtzeevalue_load(const char *root, struct yahtzeevalue_error *);
void yahtzeevalue_unload(yahtzeevalue_t *, struct yahtzeevalue_error *);
double yahtzeevalue_lookup(yahtzeevalue_t *, int state, struct yahtzeevalue_error *);
int yahtzeevalue_best_action(yahtzeevalue_t *, int state, int histogram, struct yahtzeevalue_error *);
int yahtzeevalue_keep_first(yahtzeevalue_t *, int state, int histogram, struct yahtzeevalue_error *);
int yahtzeevalue_keep_second(yahtzeevalue_t *, int state, int histogram, struct yahtzeevalue_error *);

void yahtzeevalue_free(char *);
