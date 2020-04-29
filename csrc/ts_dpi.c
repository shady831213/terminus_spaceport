#ifndef __TS_DPI_C__
#define __TS_DPI_C__
#include <ts_dpi.h>

void* tsv_new_allocator(const uint64_t base, const uint64_t size){
    return __ts_new_allocator(base, size);
}

void* tsv_new_locked_allocator(const uint64_t base, const uint64_t size) {
    return __ts_new_locked_allocator(base, size);
}

uint64_t tsv_alloc_addr(const void* allocator, const uint64_t size, const uint64_t align) {
    return __ts_alloc_addr(allocator, size, align);
}

void tsv_free_addr(const void* allocator, const uint64_t addr) {
    __ts_free_addr(allocator, addr);
}

void* tsv_space() {
    return __ts_space();
}

void tsv_delete_region(const void* space, const char* name) {
    __ts_delete_region(space, name);
}

void* tsv_get_region(const void* space, const char* name) {
    void* ptr = __ts_get_region(space, name);
    __ts_clean_region(space, name, ptr);
    return ptr;
}

void* tsv_add_region(const void* space, const char* name, void* region) {
    void* ptr = __ts_add_region(space,name, region);
    __ts_clean_region(space,name,ptr);
    __ts_clean_region(space,name,region);
    return ptr;
}

void* tsv_alloc_region(void* heap, uint64_t size, uint64_t align) {
    assert(heap != NULL);
    return __ts_alloc_region(heap, size, align, false);
}

void* tsv_root_region(uint64_t size, uint64_t align) {
    return __ts_alloc_region(NULL, size, align, false);
}

void* tsv_lazy_root_region(uint64_t size, uint64_t align) {
    return __ts_alloc_region(NULL, size, align, true);
}

void* tsv_map_region(const void* region, uint64_t base) {
    return __ts_map_region(region, base);
}

void* tsv_map_region_partial(const void* region, uint64_t base, uint64_t offset, uint64_t size) {
    return __ts_map_region_partial(region, base, offset, size);
}

void* tsv_heap(const void* region) {
    return __ts_heap(region);
}

void tsv_free_region(const void* region) {
    __ts_free_region(region);
}

void tsv_free_heap(const void* heap) {
    __ts_free_heap(heap);
}

void tsv_region_write_u8(const void* region, const uint64_t addr, const uint8_t data) {
    __ts_region_write_u8(region, addr, data);
}

void tsv_region_write_u16(const void* region, const uint64_t addr, const uint16_t data) {
    __ts_region_write_u16(region, addr, data);
}

void tsv_region_write_u32(const void* region, const uint64_t addr, const uint32_t data) {
    __ts_region_write_u32(region, addr, data);
}

void tsv_region_write_u64(const void* region, const uint64_t addr, const uint64_t data) {
    __ts_region_write_u64(region, addr, data);
}

void tsv_region_read_u8(const void* region, const uint64_t addr, uint8_t* data){
    *data = __ts_region_read_u8(region, addr);
}
void tsv_region_read_u16(const void* region, const uint64_t addr, uint16_t* data){
    *data = __ts_region_read_u16(region, addr);
}
void tsv_region_read_u32(const void* region, const uint64_t addr, uint32_t* data){
    *data = __ts_region_read_u32(region, addr);
}
void tsv_region_read_u64(const void* region, const uint64_t addr, uint64_t* data){
    *data =__ts_region_read_u64(region, addr);
}

void tsv_space_write_u8(const void* space, const uint64_t addr, const uint8_t data) {
    __ts_space_write_u8(space, addr, data);
}

void tsv_space_write_u16(const void* space, const uint64_t addr, const uint16_t data) {
    __ts_space_write_u16(space, addr, data);
}

void tsv_space_write_u32(const void* space, const uint64_t addr, const uint32_t data) {
    __ts_space_write_u32(space, addr, data);
}

void tsv_space_write_u64(const void* space, const uint64_t addr, const uint64_t data) {
    __ts_space_write_u64(space, addr, data);
}

void tsv_space_read_u8(const void* space, const uint64_t addr, uint8_t* data){
    *data = __ts_space_read_u8(space, addr);
}
void tsv_space_read_u16(const void* space, const uint64_t addr, uint16_t* data){
    *data = __ts_space_read_u16(space, addr);
}
void tsv_space_read_u32(const void* space, const uint64_t addr, uint32_t* data){
    *data = __ts_space_read_u32(space, addr);
}
void tsv_space_read_u64(const void* space, const uint64_t addr, uint64_t* data){
    *data =__ts_space_read_u64(space, addr);
}

uint64_t tsv_region_base(const void* region){
    return ((ts_mem_info*)__ts_region_info(region))->base;
}
uint64_t tsv_region_size(const void* region){
    return ((ts_mem_info*)__ts_region_info(region))->size;
}

#endif