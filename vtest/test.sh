cargo build
verilator --cc --exe -sv -o $PWD/test --vpi \
--top-module TestModule verilator_main.cc test.v +incdir+$PWD/../vsrc \
$PWD/../target/debug/libdpi_memory.a \
-CFLAGS -DVERILATOR -CFLAGS -fPIC -CFLAGS -I$PWD/../csrc -CFLAGS -I$PWD \
-LDFLAGS -Wl,-Bdynamic -LDFLAGS -lpthread -LDFLAGS -ldl -LDFLAGS -lm
make -C obj_dir -f VTestModule.mk
./test
rm test
rm -rf obj_dir