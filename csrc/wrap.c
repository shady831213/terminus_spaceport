#ifndef __DM_WRAP_C__
#define __DM_WRAP_C__
#include <wrap.h>
void* dm_get_region(void* space, char* name) {
    void* ptr = __dm_get_region(space, name);
    __dm_clean_region(space, name, ptr);
    return ptr;
}
void* dm_add_region(void* space, char* name, void* region) {
    void* ptr = __dm_add_region(space,name, region);
    __dm_clean_region(space,name,ptr);
    __dm_clean_region(space,name,region);
    return ptr;
}
#endif