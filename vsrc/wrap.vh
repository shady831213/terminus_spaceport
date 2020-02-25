`ifndef __DM_WRAP_VH__
`define __DM_WRAP_VH__
typedef struct {
    longint unsigned base;
    longint unsigned size;
} dm_mem_info;

import "DPI-C" function chandle dm_new_allocator(
    input longint unsigned base,
    input longint unsigned size
);
import "DPI-C" function chandle dm_new_locked_allocator
(
    input longint unsigned base,
    input longint unsigned size
);
import "DPI-C" function longint unsigned dm_alloc_addr
(
    input chandle  allocator,
    input longint unsigned size,
    input longint unsigned align
);
import "DPI-C" function void dm_free_addr
(
    input chandle allocator,
    input longint unsigned addr
);

import "DPI-C" function void dm_free_addr
(
    input chandle allocator,
    input longint unsigned addr
);

import "DPI-C" function chandle dm_get_space(input string name);
import "DPI-C" function chandle dm_get_region(input chandle space , input string name);
import "DPI-C" function chandle dm_alloc_region(input chandle heap, input longint unsigned size, input longint unsigned align);
import "DPI-C" function chandle dm_map_region(input chandle region, input longint unsigned base);
import "DPI-C" function chandle dm_heap(input chandle region);
import "DPI-C" function void dm_free_region(input chandle region);
import "DPI-C" function void dm_free_heap(input chandle heap);
import "DPI-C" function void dm_region_info(input chandle region, output dm_mem_info info);

`endif