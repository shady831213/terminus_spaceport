extern "C" {
#include <dm_c.h>
static void* root_space = dmc_new_space();
void* dmc_get_space(const char* name) {
    return root_space;
}
void* dmv_get_space(const char* name) {
    return root_space;
}
}