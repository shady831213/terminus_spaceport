#ifndef __TS_C_H__
#define __TS_C_H__
#include <ts_ffi.h>

void* tsc_new_allocator(const uint64_t base, const uint64_t size);
void* tsc_new_locked_allocator(const uint64_t base, const uint64_t size);
uint64_t tsc_alloc_addr(const void* allocator, const uint64_t size, const uint64_t align);
void tsc_free_addr(const void* allocator, const uint64_t addr);

void* tsc_space(const char* name);
void tsc_delete_region(const void* space, const char* name);
void* tsc_get_region(const void* space, const char* name);
void* tsc_add_region(const void* space, const char* name, void* region);

void* tsc_alloc_region(void* heap, uint64_t size, uint64_t align);
void* tsc_root_region(uint64_t size, uint64_t align);
void* tsc_lazy_root_region(uint64_t size, uint64_t align);

void* tsc_map_region(const void* region, uint64_t base);
void* tsc_map_region_partial(const void* region, uint64_t base, uint64_t offset, uint64_t size);
void* tsc_heap(const void* region);
void tsc_free_region(const void* region);
void tsc_free_heap(const void* heap);

void tsc_region_write_u8(const void* region, const uint64_t addr, const uint8_t data);
void tsc_region_write_u16(const void* region, const uint64_t addr, const uint16_t data);
void tsc_region_write_u32(const void* region, const uint64_t addr, const uint32_t data);
void tsc_region_write_u64(const void* region, const uint64_t addr, const uint64_t data);
uint8_t tsc_region_read_u8(const void* region, const uint64_t addr);
uint16_t tsc_region_read_u16(const void* region, const uint64_t addr);
uint32_t tsc_region_read_u32(const void* region, const uint64_t addr);
uint64_t tsc_region_read_u64(const void* region, const uint64_t addr);

void tsc_space_write_u8(const void* space, const uint64_t addr, const uint8_t data);
void tsc_space_write_u16(const void* space, const uint64_t addr, const uint16_t data);
void tsc_space_write_u32(const void* space, const uint64_t addr, const uint32_t data);
void tsc_space_write_u64(const void* space, const uint64_t addr, const uint64_t data);
uint8_t tsc_space_read_u8(const void* space, const uint64_t addr);
uint16_t tsc_space_read_u16(const void* space, const uint64_t addr);
uint32_t tsc_space_read_u32(const void* space, const uint64_t addr);
uint64_t tsc_space_read_u64(const void* space, const uint64_t addr);


ts_mem_info* tsc_region_info(const void* region);

#endif