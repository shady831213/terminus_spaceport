#ifndef __TS_FFI_H__
#define __TS_FFI_H__
#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>
#include <assert.h>
typedef struct{
    uint64_t base;
    uint64_t size;
} ts_mem_info ;

extern void* __ts_new_allocator(const uint64_t base, const uint64_t size);
extern void* __ts_new_locked_allocator(const uint64_t base, const uint64_t size);
extern uint64_t __ts_alloc_addr(const void* allocator, const uint64_t size, const uint64_t align);
extern void __ts_free_addr(const void* allocator, const uint64_t addr);

extern void* __ts_space(const char* name);
extern void* __ts_add_region(const void* space, const char* name, void* region);
extern void __ts_clean_region(const void* space, const char* name, void* ptr);
extern void* __ts_get_region(const void* space, const char* name);
extern void __ts_delete_region(const void* space, const char* name);

extern void* __ts_alloc_region(void* heap, uint64_t size, uint64_t align, bool lazy);
extern void* __ts_map_region(const void* region, uint64_t base);
extern void* __ts_map_region_partial(const void* region, uint64_t base, uint64_t offset, uint64_t size);
extern void* __ts_heap(const void* region);
extern void __ts_free_region(const void* region);
extern void __ts_free_heap(const void* heap);
extern void* __ts_region_info(const void* region);

extern void __ts_region_write_u8(const void* region, const uint64_t addr, const uint8_t data);
extern void __ts_region_write_u16(const void* region, const uint64_t addr, const uint16_t data);
extern void __ts_region_write_u32(const void* region, const uint64_t addr, const uint32_t data);
extern void __ts_region_write_u64(const void* region, const uint64_t addr, const uint64_t data);
extern uint8_t __ts_region_read_u8(const void* region, const uint64_t addr);
extern uint16_t __ts_region_read_u16(const void* region, const uint64_t addr);
extern uint32_t __ts_region_read_u32(const void* region, const uint64_t addr);
extern uint64_t __ts_region_read_u64(const void* region, const uint64_t addr);

extern void __ts_space_write_u8(const void* space, const uint64_t addr, const uint8_t data);
extern void __ts_space_write_u16(const void* space, const uint64_t addr, const uint16_t data);
extern void __ts_space_write_u32(const void* space, const uint64_t addr, const uint32_t data);
extern void __ts_space_write_u64(const void* space, const uint64_t addr, const uint64_t data);
extern uint8_t __ts_space_read_u8(const void* space, const uint64_t addr);
extern uint16_t __ts_space_read_u16(const void* space, const uint64_t addr);
extern uint32_t __ts_space_read_u32(const void* space, const uint64_t addr);
extern uint64_t __ts_space_read_u64(const void* space, const uint64_t addr);

#endif