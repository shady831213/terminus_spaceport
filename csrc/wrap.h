#include <stdint.h>
#include <stdio.h>
extern void* new_allocator(uint64_t base, uint64_t size);
extern uint64_t alloc(void* allocator, uint64_t size, uint64_t align);
extern void* new_locked_allocator(uint64_t base, uint64_t size);
extern uint64_t locked_alloc(void* allocator, uint64_t size, uint64_t align);

