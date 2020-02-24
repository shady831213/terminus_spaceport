#include "wrap.h"
int main() {
    void* space = dm_new_space();
    void* region = dm_alloc_region(NULL, 8, 1);
    dm_add_region(space, "region", region);
    void* heap = dm_heap(region);
    void* region2 = dm_alloc_region(heap, 2,1);
    void* region3 = dm_alloc_region(heap, 2,1);
    void* region5 = dm_get_region(space, "region");
    void* region4 = dm_map_region(region5, 10);
    dm_delete_region(space, "region");
    dm_free_region(region3);
    dm_free_region(region);
    dm_free_heap(heap);
    dm_free_region(region4);
    dm_free_region(region2);
}