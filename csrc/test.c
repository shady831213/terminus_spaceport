#include "wrap.h"
int main() {
    void* allocator = new_allocator(1, 9);
    printf("addr = 0x%lu\n", alloc(allocator, 1, 4));
    printf("addr = 0x%lu\n", alloc(allocator, 1, 4));
    void* locked_allocator = new_locked_allocator(1, 9);
    printf("laddr = 0x%lu\n", locked_alloc(locked_allocator, 1, 1));
    printf("laddr = 0x%lu\n", locked_alloc(locked_allocator, 1, 1));
}