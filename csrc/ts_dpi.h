#ifndef __TS_DPI_H__
#define __TS_DPI_H__
#include <ts_ffi.h>

void* tsv__new_allocator(const uint64_t base, const uint64_t size);
void* tsv__new_locked_allocator(const uint64_t base, const uint64_t size);
uint64_t tsv__alloc_addr(const void* allocator, const uint64_t size, const uint64_t align);
void tsv__free_addr(const void* allocator, const uint64_t addr);

void* tsv__space(const char* name);
void tsv__delete_region(const void* space, const char* name);
void* tsv__get_region(const void* space, const char* name);
void* tsv__add_region(const void* space, const char* name, void* region);

void* tsv__alloc_region(void* heap, uint64_t size, uint64_t align);
void* tsv__map_region(const void* region, uint64_t base);
void* tsv__heap(const void* region);
void tsv__free_region(const void* region);
void tsv__free_heap(const void* heap);

void tsv__region_write_u8(const void* region, const uint64_t addr, const uint8_t data);
void tsv__region_write_u16(const void* region, const uint64_t addr, const uint16_t data);
void tsv__region_write_u32(const void* region, const uint64_t addr, const uint32_t data);
void tsv__region_write_u64(const void* region, const uint64_t addr, const uint64_t data);
void tsv__region_read_u8(const void* heap, const uint64_t addr, uint8_t* data);
void tsv__region_read_u16(const void* heap, const uint64_t addr, uint16_t* data);
void tsv__region_read_u32(const void* heap, const uint64_t addr, uint32_t* data);
void tsv__region_read_u64(const void* heap, const uint64_t addr, uint64_t* data);

uint64_t tsv__region_base(const void* region);
uint64_t tsv__region_size(const void* region);

#endif