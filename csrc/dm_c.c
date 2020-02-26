#ifndef __DM_C_C__
#define __DM_C_C__
#include <dm_c.h>

void* dmc_new_allocator(const uint64_t base, const uint64_t size){
    return __dm_new_allocator(base, size);
}

void* dmc_new_locked_allocator(const uint64_t base, const uint64_t size) {
    return __dm_new_locked_allocator(base, size);
}

uint64_t dmc_alloc_addr(const void* allocator, const uint64_t size, const uint64_t align) {
    return __dm_alloc_addr(allocator, size, align);
}

void dmc_free_addr(const void* allocator, const uint64_t addr) {
    __dm_free_addr(allocator, addr);
}

void* dmc_new_space() {
    return __dm_new_space();
}

void dmc_delete_region(const void* space, const char* name) {
    __dm_delete_region(space, name);
}

void* dmc_get_region(const void* space, const char* name) {
    void* ptr = __dm_get_region(space, name);
    __dm_clean_region(space, name, ptr);
    return ptr;
}

void* dmc_add_region(const void* space, const char* name, void* region) {
    void* ptr = __dm_add_region(space,name, region);
    __dm_clean_region(space,name,ptr);
    __dm_clean_region(space,name,region);
    return ptr;
}

void* dmc_alloc_region(void* heap, uint64_t size, uint64_t align) {
    return __dm_alloc_region(heap, size, align);
}

void* dmc_map_region(const void* region, uint64_t base) {
    return __dm_map_region(region, base);
}

void* dmc_heap(const void* region) {
    return __dm_heap(region);
}

void dmc_free_region(const void* region) {
    __dm_free_region(region);
}

void dmc_free_heap(const void* heap) {
    __dm_free_heap(heap);
}

void dmc_region_write_u8(const void* region, const uint64_t addr, const uint8_t data) {
    __dm_region_write_u8(region, addr, data);
}

void dmc_region_write_u16(const void* region, const uint64_t addr, const uint16_t data) {
    __dm_region_write_u16(region, addr, data);
}

void dmc_region_write_u32(const void* region, const uint64_t addr, const uint32_t data) {
    __dm_region_write_u32(region, addr, data);
}

void dmc_region_write_u64(const void* region, const uint64_t addr, const uint64_t data) {
    __dm_region_write_u64(region, addr, data);
}

uint8_t dmc_region_read_u8(const void* region, const uint64_t addr){
    return __dm_region_read_u8(region, addr);
}
uint16_t dmc_region_read_u16(const void* region, const uint64_t addr){
    return __dm_region_read_u16(region, addr);
}
uint32_t dmc_region_read_u32(const void* region, const uint64_t addr){
    return __dm_region_read_u32(region, addr);
}
uint64_t dmc_region_read_u64(const void* region, const uint64_t addr){
    return __dm_region_read_u64(region, addr);
}

dm_mem_info* dmc_region_info(const void* region){
    return (dm_mem_info*)__dm_region_info(region);
}

#endif