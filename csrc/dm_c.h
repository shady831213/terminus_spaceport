#ifndef __DM_C_H__
#define __DM_C_H__
#include <dm_ffi.h>

void* dmc_new_allocator(const uint64_t base, const uint64_t size);
void* dmc_new_locked_allocator(const uint64_t base, const uint64_t size);
uint64_t dmc_alloc_addr(const void* allocator, const uint64_t size, const uint64_t align);
void dmc_free_addr(const void* allocator, const uint64_t addr);

void* dmc_space(const char* name);
void dmc_delete_region(const void* space, const char* name);
void* dmc_get_region(const void* space, const char* name);
void* dmc_add_region(const void* space, const char* name, void* region);

void* dmc_alloc_region(void* heap, uint64_t size, uint64_t align);
void* dmc_map_region(const void* region, uint64_t base);
void* dmc_heap(const void* region);
void dmc_free_region(const void* region);
void dmc_free_heap(const void* heap);

void dmc_region_write_u8(const void* region, const uint64_t addr, const uint8_t data);
void dmc_region_write_u16(const void* region, const uint64_t addr, const uint16_t data);
void dmc_region_write_u32(const void* region, const uint64_t addr, const uint32_t data);
void dmc_region_write_u64(const void* region, const uint64_t addr, const uint64_t data);
uint8_t dmc_region_read_u8(const void* region, const uint64_t addr);
uint16_t dmc_region_read_u16(const void* region, const uint64_t addr);
uint32_t dmc_region_read_u32(const void* region, const uint64_t addr);
uint64_t dmc_region_read_u64(const void* region, const uint64_t addr);

dm_mem_info* dmc_region_info(const void* region);

#endif