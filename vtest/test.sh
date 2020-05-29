CARGO_TARGET_DIR=${PWD}/../target cargo build
verilator --cc --exe -sv -o $PWD/test --vpi \
--top-module TestModule verilator_main.cc test.v +incdir+$PWD/../target/debug \
$PWD/../target/debug/libterminus_spaceport.a \
-CFLAGS -DVERILATOR -CFLAGS -fPIC -CFLAGS -I$PWD/../target/debug -CFLAGS -I$PWD \
-LDFLAGS -Wl,-Bdynamic -LDFLAGS -lpthread -LDFLAGS -ldl -LDFLAGS -lm -LDFLAGS -lrt
make -C obj_dir -f VTestModule.mk
./test
rm test
rm -rf obj_dir