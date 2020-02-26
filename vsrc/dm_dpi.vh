`ifndef __DM_DPI_VH__
`define __DM_DPI_VH__
import "DPI-C" function chandle dmv_new_allocator(
    input longint unsigned base,
    input longint unsigned size
);
import "DPI-C" function chandle dmv_new_locked_allocator
(
    input longint unsigned base,
    input longint unsigned size
);
import "DPI-C" function longint unsigned dmv_alloc_addr
(
    input chandle  allocator,
    input longint unsigned size,
    input longint unsigned align
);
import "DPI-C" function void dmv_free_addr
(
    input chandle allocator,
    input longint unsigned addr
);


import "DPI-C" function chandle dmv_space(input string name);
import "DPI-C" function chandle dmv_get_region(input chandle space , input string name);
import "DPI-C" function chandle dmv_alloc_region(input chandle heap, input longint unsigned size, input longint unsigned align);
import "DPI-C" function chandle dmv_map_region(input chandle region, input longint unsigned base);
import "DPI-C" function chandle dmv_heap(input chandle region);
import "DPI-C" function void dmv_free_region(input chandle region);
import "DPI-C" function void dmv_free_heap(input chandle heap);
import "DPI-C" function longint unsigned dmv_region_base(input chandle region);
import "DPI-C" function longint unsigned dmv_region_size(input chandle region);

import "DPI-C" function void dmv_region_write_u8(input chandle  region, input longint unsigned addr, input byte unsigned data);
import "DPI-C" function void dmv_region_write_u16(input chandle  region, input longint unsigned addr, input shortint unsigned data);
import "DPI-C" function void dmv_region_write_u32(input chandle  region, input longint unsigned addr, input int unsigned data);
import "DPI-C" function void dmv_region_write_u64(input chandle  region, input longint unsigned addr, input longint unsigned data);
import "DPI-C" function void dmv_region_read_u8(input chandle  region, input longint unsigned addr, output byte unsigned data);
import "DPI-C" function void dmv_region_read_u16(input chandle  region, input longint unsigned addr, output shortint unsigned data);
import "DPI-C" function void dmv_region_read_u32(input chandle  region, input longint unsigned addr, output int unsigned data);
import "DPI-C" function void dmv_region_read_u64(input chandle  region, input longint unsigned addr, output longint unsigned data);
`endif