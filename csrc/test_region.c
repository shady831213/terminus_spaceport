#include "wrap.h"
int main() {
    void* region = dm_alloc_region(NULL, 8, 1);
    void* heap = dm_heap(region);
    void* region2 = dm_alloc_region(heap, 2,1);
    void* region3 = dm_alloc_region(heap, 2,1);
    dm_free_region(region3);
    dm_free_region(region);
    dm_free_heap(heap);
    dm_free_region(region2);
}