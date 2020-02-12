cargo build
#gcc -g -o test test.c -ldpi_memory -L../target/debug -Wl,-rpath=../target/debug
gcc -g -o test test.c -L../target/debug -Wl,-Bstatic -ldpi_memory  -Wl,-Bdynamic -lpthread -ldl
./test
rm test