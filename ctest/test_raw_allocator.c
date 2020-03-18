#include <ts_c.h>
int main() {
    void* allocator = tsc_new_allocator(1, 9);
    void* locked_allocator = tsc_new_locked_allocator(1, 9);

    uint64_t addr = tsc_alloc_addr(allocator, 1, 4);
    printf("addr = %lu\n", addr);
    tsc_free_addr(allocator, addr);
    printf("addr = %lu\n", tsc_alloc_addr(allocator, 1, 4));

    uint64_t laddr = tsc_alloc_addr(locked_allocator, 1, 1);
    printf("laddr = %lu\n", laddr);

    printf("addr = %lu\n", tsc_alloc_addr(allocator, 1, 4));

    tsc_free_addr(locked_allocator, laddr);
    printf("laddr = %lu\n", tsc_alloc_addr(locked_allocator, 1, 1));
    printf("laddr = %lu\n", tsc_alloc_addr(locked_allocator, 1, 1));
}