#ifndef __TS_C_C__
#define __TS_C_C__
#include <ts_c.h>

void* tsc_new_allocator(const uint64_t base, const uint64_t size){
    return __ts_new_allocator(base, size);
}

void* tsc_new_locked_allocator(const uint64_t base, const uint64_t size) {
    return __ts_new_locked_allocator(base, size);
}

uint64_t tsc_alloc_addr(const void* allocator, const uint64_t size, const uint64_t align) {
    return __ts_alloc_addr(allocator, size, align);
}

void tsc_free_addr(const void* allocator, const uint64_t addr) {
    __ts_free_addr(allocator, addr);
}

void* tsc_space(const char* name) {
    return __ts_space(name);
}

void tsc_delete_region(const void* space, const char* name) {
    __ts_delete_region(space, name);
}

void* tsc_get_region(const void* space, const char* name) {
    void* ptr = __ts_get_region(space, name);
    __ts_clean_region(space, name, ptr);
    return ptr;
}

void* tsc_add_region(const void* space, const char* name, void* region) {
    void* ptr = __ts_add_region(space,name, region);
    __ts_clean_region(space,name,ptr);
    __ts_clean_region(space,name,region);
    return ptr;
}

void* tsc_alloc_region(void* heap, uint64_t size, uint64_t align) {
    return __ts_alloc_region(heap, size, align);
}

void* tsc_map_region(const void* region, uint64_t base) {
    return __ts_map_region(region, base);
}

void* tsc_heap(const void* region) {
    return __ts_heap(region);
}

void tsc_free_region(const void* region) {
    __ts_free_region(region);
}

void tsc_free_heap(const void* heap) {
    __ts_free_heap(heap);
}

void tsc_region_write_u8(const void* region, const uint64_t addr, const uint8_t data) {
    __ts_region_write_u8(region, addr, data);
}

void tsc_region_write_u16(const void* region, const uint64_t addr, const uint16_t data) {
    __ts_region_write_u16(region, addr, data);
}

void tsc_region_write_u32(const void* region, const uint64_t addr, const uint32_t data) {
    __ts_region_write_u32(region, addr, data);
}

void tsc_region_write_u64(const void* region, const uint64_t addr, const uint64_t data) {
    __ts_region_write_u64(region, addr, data);
}

uint8_t tsc_region_read_u8(const void* region, const uint64_t addr){
    return __ts_region_read_u8(region, addr);
}
uint16_t tsc_region_read_u16(const void* region, const uint64_t addr){
    return __ts_region_read_u16(region, addr);
}
uint32_t tsc_region_read_u32(const void* region, const uint64_t addr){
    return __ts_region_read_u32(region, addr);
}
uint64_t tsc_region_read_u64(const void* region, const uint64_t addr){
    return __ts_region_read_u64(region, addr);
}

ts_mem_info* tsc_region_info(const void* region){
    return (ts_mem_info*)__ts_region_info(region);
}

#endif