cargo build
verilator --cc --exe -sv -o $PWD/test --vpi \
--top-module TestModule verilator_main.cc test.v \
$PWD/../target/debug/libdpi_memory.so \
-CFLAGS -DVERILATOR -CFLAGS -fPIC \
-LDFLAGS -Wl,-rpath=$PWD/../target/debug
make -C obj_dir -f VTestModule.mk
./test
rm test
rm -rf obj_dir