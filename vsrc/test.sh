make -C ../csrc -f wrap.mk
verilator --cc --exe -sv -o $PWD/test --vpi \
--top-module TestModule verilator_main.cc test.v \
$PWD/../csrc/libdpi_memory.c.so \
-CFLAGS -DVERILATOR -CFLAGS -fPIC \
-LDFLAGS -Wl,-rpath=$PWD/../csrc
make -C obj_dir -f VTestModule.mk
./test
rm test
rm -rf obj_dir