`ifndef __DM_WRAP_VH__
`define __DM_WRAP_VH__
import "DPI-C" function chandle dm_new_allocator(
    input longint unsigned base,
    input longint unsigned size
);
import "DPI-C" function chandle dm_new_locked_allocator
(
    input longint unsigned base,
    input longint unsigned size
);
import "DPI-C" function longint unsigned dm_alloc
(
    input chandle  allocator,
    input longint unsigned size,
    input longint unsigned align
);
import "DPI-C" function void dm_free
(
    input chandle allocator,
    input longint unsigned addr
);
`endif