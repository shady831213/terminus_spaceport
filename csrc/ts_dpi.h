#ifndef __TS_DPI_H__
#define __TS_DPI_H__
#include <ts_ffi.h>

void* tsv_new_allocator(const uint64_t base, const uint64_t size);
void* tsv_new_locked_allocator(const uint64_t base, const uint64_t size);
uint64_t tsv_alloc_addr(const void* allocator, const uint64_t size, const uint64_t align);
void tsv_free_addr(const void* allocator, const uint64_t addr);

void* tsv_space(const char* name);
void tsv_delete_region(const void* space, const char* name);
void* tsv_get_region(const void* space, const char* name);
void* tsv_add_region(const void* space, const char* name, void* region);

void* tsv_alloc_region(void* heap, uint64_t size, uint64_t align);
void* tsv_root_region(uint64_t size, uint64_t align);
void* tsv_lazy_root_region(uint64_t size, uint64_t align);

void* tsv_map_region(const void* region, uint64_t base);
void* tsv_map_region_partial(const void* region, uint64_t base, uint64_t offset, uint64_t size);
void* tsv_heap(const void* region);
void tsv_free_region(const void* region);
void tsv_free_heap(const void* heap);

void tsv_region_write_u8(const void* region, const uint64_t addr, const uint8_t data);
void tsv_region_write_u16(const void* region, const uint64_t addr, const uint16_t data);
void tsv_region_write_u32(const void* region, const uint64_t addr, const uint32_t data);
void tsv_region_write_u64(const void* region, const uint64_t addr, const uint64_t data);
void tsv_region_read_u8(const void* heap, const uint64_t addr, uint8_t* data);
void tsv_region_read_u16(const void* heap, const uint64_t addr, uint16_t* data);
void tsv_region_read_u32(const void* heap, const uint64_t addr, uint32_t* data);
void tsv_region_read_u64(const void* heap, const uint64_t addr, uint64_t* data);

uint64_t tsv_region_base(const void* region);
uint64_t tsv_region_size(const void* region);

#endif