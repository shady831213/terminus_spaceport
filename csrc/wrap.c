#ifndef __DM_WRAP_C__
#define __DM_WRAP_C__
#include <wrap.h>
void* dm_get_region(const void* space, char* name) {
    void* ptr = __dm_get_region(space, name);
    __dm_clean_region(space, name, ptr);
    return ptr;
}
void* dm_add_region(const void* space, char* name, void* region) {
    void* ptr = __dm_add_region(space,name, region);
    __dm_clean_region(space,name,ptr);
    __dm_clean_region(space,name,region);
    return ptr;
}

uint8_t dm_c_region_read_u8(const void* heap, const uint64_t addr){
    return __dm_region_read_u8(heap, addr);
}
uint16_t dm_c_region_read_u16(const void* heap, const uint64_t addr){
    return __dm_region_read_u16(heap, addr);
}
uint32_t dm_c_region_read_u32(const void* heap, const uint64_t addr){
    return __dm_region_read_u32(heap, addr);
}
uint64_t dm_c_region_read_u64(const void* heap, const uint64_t addr){
    return __dm_region_read_u64(heap, addr);
}
void dm_dpi_region_read_u8(const void* heap, const uint64_t addr, uint8_t* data){
    uint8_t _data = __dm_region_read_u8(heap, addr);
    *data = data;
}
void dm_dpi_region_read_u16(const void* heap, const uint64_t addr, uint16_t* data){
    uint16_t _data = __dm_region_read_u16(heap, addr);
    *data = data;
}
void dm_dpi_region_read_u32(const void* heap, const uint64_t addr, uint32_t* data){
    uint32_t _data = __dm_region_read_u32(heap, addr);
    *data = data;
}
void dm_dpi_region_read_u64(const void* heap, const uint64_t addr, uint64_t* data){
    uint64_t _data = __dm_region_read_u64(heap, addr);
    *data = data;
}

dm_mem_info* dm_c_region_info(const void* region){
    return (dm_mem_info*)__dm_region_info(region);
}

void dm_dpi_region_info(const void* region, dm_mem_info* info){
    dm_mem_info* ptr = (dm_mem_info*)__dm_region_info(region);
    info->base = ptr->base;
    info->size = ptr->size;
}
#endif