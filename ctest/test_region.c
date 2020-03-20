#include <ts_c.h>
int main() {
    void* space = tsc_space("root");
    void* region = tsc_add_region(space, "region", tsc_lazy_root_region(8, 1));
    void* heap = tsc_heap(region);
    void* region2 = tsc_alloc_region(heap, 2,1);
    void* region3 = tsc_alloc_region(heap, 2,1);
    void* region4 = tsc_map_region(tsc_get_region(space, "region"), 10);
    printf("region base = %lu; size = %lu\n", tsc_region_info(tsc_get_region(space, "region"))->base, tsc_region_info(tsc_get_region(space, "region"))->size);
    tsc_region_write_u16(tsc_get_region(space, "region"), tsc_region_info(region)->base, 0x5aa5);
    printf("region4 base = %lu; size = %lu\n", tsc_region_info(region4)->base, tsc_region_info(region4)->size);
    printf("region4 addr = %lu; data = %x\n", tsc_region_info(region4)->base, tsc_region_read_u8(region4, tsc_region_info(region4)->base));
    printf("region4 addr = %lu; data = %x\n", tsc_region_info(region4)->base+1, tsc_region_read_u8(region4, tsc_region_info(region4)->base+1));
    tsc_delete_region(space, "region");
    tsc_free_region(region3);
    tsc_free_region(region);
    tsc_free_heap(heap);
    tsc_free_region(region4);
    tsc_free_region(region2);
}