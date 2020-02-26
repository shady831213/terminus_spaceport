#ifndef __DM_DPI_H__
#define __DM_DPI_H__
#include <dm_ffi.h>

void* dmv_new_allocator(const uint64_t base, const uint64_t size);
void* dmv_new_locked_allocator(const uint64_t base, const uint64_t size);
uint64_t dmv_alloc_addr(const void* allocator, const uint64_t size, const uint64_t align);
void dmv_free_addr(const void* allocator, const uint64_t addr);

void* dmv_new_space();
void dmv_delete_region(const void* space, const char* name);
void* dmv_get_region(const void* space, const char* name);
void* dmv_add_region(const void* space, const char* name, void* region);

void* dmv_alloc_region(void* heap, uint64_t size, uint64_t align);
void* dmv_map_region(const void* region, uint64_t base);
void* dmv_heap(const void* region);
void dmv_free_region(const void* region);
void dmv_free_heap(const void* heap);

void dmv_region_write_u8(const void* region, const uint64_t addr, const uint8_t data);
void dmv_region_write_u16(const void* region, const uint64_t addr, const uint16_t data);
void dmv_region_write_u32(const void* region, const uint64_t addr, const uint32_t data);
void dmv_region_write_u64(const void* region, const uint64_t addr, const uint64_t data);
void dmv_region_read_u8(const void* heap, const uint64_t addr, uint8_t* data);
void dmv_region_read_u16(const void* heap, const uint64_t addr, uint16_t* data);
void dmv_region_read_u32(const void* heap, const uint64_t addr, uint32_t* data);
void dmv_region_read_u64(const void* heap, const uint64_t addr, uint64_t* data);

uint64_t dmv_region_base(const void* region);
uint64_t dmv_region_size(const void* region);

#endif