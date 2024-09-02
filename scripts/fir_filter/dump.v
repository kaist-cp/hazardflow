module dump();

initial begin
    $dumpfile("fir.vcd");
    $dumpvars(0, fir);
end

endmodule
