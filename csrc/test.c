#include "wrap.h"
int main() {
    void* allocator = dm_new_allocator(1, 9);
    uint64_t addr = dm_alloc(allocator, 1, 4);
    printf("addr = 0x%lu\n", addr);
    dm_free(allocator, addr);
    printf("addr = 0x%lu\n", dm_alloc(allocator, 1, 4));
    printf("addr = 0x%lu\n", dm_alloc(allocator, 1, 4));

    void* locked_allocator = dm_new_locked_allocator(1, 9);
    uint64_t laddr = dm_locked_alloc(locked_allocator, 1, 1);
    printf("laddr = 0x%lu\n", laddr);
    dm_locked_free(locked_allocator, laddr);
    printf("laddr = 0x%lu\n", dm_locked_alloc(locked_allocator, 1, 1));
    printf("laddr = 0x%lu\n", dm_locked_alloc(locked_allocator, 1, 1));
}