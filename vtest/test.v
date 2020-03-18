`include "ts_dpi.vh";
module TestModule(input bit clock);
chandle allocator;
reg [3:0]cnt;
initial begin
    chandle main_memory, rom;
    bit[31:0] data;
    main_memory = tsv_get_region(tsv_space("root"), "main_memory");
    rom = tsv_get_region(tsv_space("root"), "rom");
    $display("read rom @0x%0x", tsv_region_base(rom));
    tsv_region_read_u16(rom, tsv_region_base(rom), data[15:0]);
    tsv_region_read_u16(rom, tsv_region_base(rom)+2, data[31:16]);
    $display("read rom @0x%0x data=0x%0x", tsv_region_base(rom), data);
    tsv_region_write_u32(main_memory,tsv_region_base(main_memory), data);
    allocator = tsv_new_allocator(1, 16);
    cnt = 0;
end

always @(posedge clock) begin
    cnt <= cnt + 1;
end

always @(posedge clock) begin
    if (cnt[3]) begin
        tsv_free_addr(allocator, {61'b0, cnt[2:0]} + 1);
    end
    else begin
        $display("alloc addr = 0x%0x", tsv_alloc_addr(allocator, 1, 1));
    end
end

always @(posedge clock) begin
    if (&cnt) begin
        $finish();
    end
end
endmodule