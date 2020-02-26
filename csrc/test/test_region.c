#include <dm_c.h>
int main() {
    void* space = dmc_new_space();
    void* region = dmc_add_region(space, "region", dmc_alloc_region(NULL, 8, 1));
    void* heap = dmc_heap(region);
    void* region2 = dmc_alloc_region(heap, 2,1);
    void* region3 = dmc_alloc_region(heap, 2,1);
    void* region4 = dmc_map_region(dmc_get_region(space, "region"), 10);
    printf("region base = %lu; size = %lu\n", dmc_region_info(dmc_get_region(space, "region"))->base, dmc_region_info(dmc_get_region(space, "region"))->size);
    dmc_region_write_u16(dmc_get_region(space, "region"), dmc_region_info(region)->base, 0x5aa5);
    printf("region4 base = %lu; size = %lu\n", dmc_region_info(region4)->base, dmc_region_info(region4)->size);
    printf("region4 addr = %lu; data = %x\n", dmc_region_info(region4)->base, dmc_region_read_u8(region4, dmc_region_info(region4)->base));
    printf("region4 addr = %lu; data = %x\n", dmc_region_info(region4)->base+1, dmc_region_read_u8(region4, dmc_region_info(region4)->base+1));
    dmc_delete_region(space, "region");
    dmc_free_region(region3);
    dmc_free_region(region);
    dmc_free_heap(heap);
    dmc_free_region(region4);
    dmc_free_region(region2);
}