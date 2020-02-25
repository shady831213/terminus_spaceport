`include "wrap.vh";
module TestModule(input bit clock);
chandle allocator;
reg [3:0]cnt;


initial begin
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