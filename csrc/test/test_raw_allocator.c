#include <dm_c.h>
int main() {
    void* allocator = dmc_new_allocator(1, 9);
    void* locked_allocator = dmc_new_locked_allocator(1, 9);

    uint64_t addr = dmc_alloc_addr(allocator, 1, 4);
    printf("addr = %lu\n", addr);
    dmc_free_addr(allocator, addr);
    printf("addr = %lu\n", dmc_alloc_addr(allocator, 1, 4));

    uint64_t laddr = dmc_alloc_addr(locked_allocator, 1, 1);
    printf("laddr = %lu\n", laddr);

    printf("addr = %lu\n", dmc_alloc_addr(allocator, 1, 4));

    dmc_free_addr(locked_allocator, laddr);
    printf("laddr = %lu\n", dmc_alloc_addr(locked_allocator, 1, 1));
    printf("laddr = %lu\n", dmc_alloc_addr(locked_allocator, 1, 1));
}