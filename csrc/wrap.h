#ifndef __DM_WRAP_H__
#define __DM_WRAP_H__
#include <stdint.h>
#include <stdio.h>
extern void* dm_new_allocator(uint64_t base, uint64_t size);
extern void* dm_new_locked_allocator(uint64_t base, uint64_t size);
extern uint64_t dm_alloc_addr(void* allocator, uint64_t size, uint64_t align);
extern void dm_free_addr(void* allocator, uint64_t addr);

extern void* dm_alloc_region(void* heap, uint64_t size, uint64_t align);
extern void* dm_heap(void* region);
extern void dm_free_region(void* region);
extern void dm_free_heap(void* heap);
#endif

