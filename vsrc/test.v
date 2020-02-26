`include "wrap.vh";
module TestModule(input bit clock);
chandle allocator;
reg [3:0]cnt;
initial begin
    chandle main_memory, rom;
    bit[31:0] data;
    dm_mem_info main_memory_info, rom_info;
    main_memory = dm_get_region(dm_get_space(""), "main_memory");
    dm_dpi_region_info(main_memory, main_memory_info);
    rom = dm_get_region(dm_get_space(""), "rom");
    dm_dpi_region_info(rom, rom_info);
    dm_dpi_region_read_u16(rom, rom_info.base, data[15:0]);
    dm_dpi_region_read_u16(rom, rom_info.base+2, data[31:16]);
    $display("read rom @0x%0x data=0x%0x", rom_info.base, data);
    dm_region_write_u32(main_memory, main_memory_info.base, data);
    allocator = dm_new_allocator(1, 16);
    cnt = 0;
end

always @(posedge clock) begin
    cnt <= cnt + 1;
end

always @(posedge clock) begin
    if (cnt[3]) begin
        dm_free_addr(allocator, {61'b0, cnt[2:0]} + 1);
    end
    else begin
        $display("alloc addr = 0x%0x", dm_alloc_addr(allocator, 1, 1));
    end
end

always @(posedge clock) begin
    if (&cnt) begin
        $finish();
    end
end
endmodule