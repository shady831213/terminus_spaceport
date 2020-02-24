#include "wrap.h"
int main() {
    void* allocator = dm_new_allocator(1, 9);
    void* locked_allocator = dm_new_locked_allocator(1, 9);

    uint64_t addr = dm_alloc_addr(allocator, 1, 4);
    printf("addr = %lu\n", addr);
    dm_free_addr(allocator, addr);
    printf("addr = %lu\n", dm_alloc_addr(allocator, 1, 4));

    uint64_t laddr = dm_alloc_addr(locked_allocator, 1, 1);
    printf("laddr = %lu\n", laddr);

    printf("addr = %lu\n", dm_alloc_addr(allocator, 1, 4));

    dm_free_addr(locked_allocator, laddr);
    printf("laddr = %lu\n", dm_alloc_addr(locked_allocator, 1, 1));
    printf("laddr = %lu\n", dm_alloc_addr(locked_allocator, 1, 1));
}