#include "wrap.h"
int main() {
    void* allocator = new_allocator(1, 9);
    uint64_t addr = alloc(allocator, 1, 4);
    printf("addr = 0x%lu\n", addr);
    void* locked_allocator = new_locked_allocator(1, 9);
    uint64_t laddr = locked_alloc(locked_allocator, 1, 8);
    printf("laddr = 0x%lu\n", laddr);
}