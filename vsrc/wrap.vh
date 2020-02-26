`ifndef __DM_WRAP_VH__
`define __DM_WRAP_VH__
typedef struct packed {
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


import "DPI-C" function chandle dm_get_space(input string name);
import "DPI-C" function chandle dm_get_region(input chandle space , input string name);
import "DPI-C" function chandle dm_alloc_region(input chandle heap, input longint unsigned size, input longint unsigned align);
import "DPI-C" function chandle dm_map_region(input chandle region, input longint unsigned base);
import "DPI-C" function chandle dm_heap(input chandle region);
import "DPI-C" function void dm_free_region(input chandle region);
import "DPI-C" function void dm_free_heap(input chandle heap);
import "DPI-C" function void dm_dpi_region_info(input chandle region, output dm_mem_info info);


import "DPI-C" function void dm_region_write_u8(input chandle  heap, input longint unsigned addr, input byte unsigned data);
import "DPI-C" function void dm_region_write_u16(input chandle  heap, input longint unsigned addr, input shortint unsigned data);
import "DPI-C" function void dm_region_write_u32(input chandle  heap, input longint unsigned addr, input int unsigned data);
import "DPI-C" function void dm_region_write_u64(input chandle  heap, input longint unsigned addr, input longint unsigned data);

import "DPI-C" function void dm_dpi_region_read_u8(input chandle  heap, input longint unsigned addr, output byte unsigned data);
import "DPI-C" function void dm_dpi_region_read_u16(input chandle  heap, input longint unsigned addr, output shortint unsigned data);
import "DPI-C" function void dm_dpi_region_read_u32(input chandle  heap, input longint unsigned addr, output int unsigned data);
import "DPI-C" function void dm_dpi_region_read_u64(input chandle  heap, input longint unsigned addr, output longint unsigned data);
`endif