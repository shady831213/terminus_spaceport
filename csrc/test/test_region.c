#include <wrap.h>
int main() {
    void* space = dm_new_space();
    void* region = dm_add_region(space, "region", dm_alloc_region(NULL, 8, 1));
    void* heap = dm_heap(region);
    void* region2 = dm_alloc_region(heap, 2,1);
    void* region3 = dm_alloc_region(heap, 2,1);
    void* region4 = dm_map_region(dm_get_region(space, "region"), 10);
    printf("region base = %lu; size = %lu\n", dm_c_region_info(dm_get_region(space, "region"))->base, dm_c_region_info(dm_get_region(space, "region"))->size);
    dm_region_write_u16(dm_get_region(space, "region"), dm_c_region_info(region)->base, 0x5aa5);
    printf("region4 base = %lu; size = %lu\n", dm_c_region_info(region4)->base, dm_c_region_info(region4)->size);
    printf("region4 addr = %lu; data = %x\n", dm_c_region_info(region4)->base, dm_c_region_read_u8(region4, dm_c_region_info(region4)->base));
    printf("region4 addr = %lu; data = %x\n", dm_c_region_info(region4)->base+1, dm_c_region_read_u8(region4, dm_c_region_info(region4)->base+1));
    dm_delete_region(space, "region");
    dm_free_region(region3);
    dm_free_region(region);
    dm_free_heap(heap);
    dm_free_region(region4);
    dm_free_region(region2);
}