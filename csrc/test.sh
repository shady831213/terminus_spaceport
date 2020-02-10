cargo build
gcc -g -o test test.c -ldpi_memory -L../target/debug -Wl,-rpath=../target/debug
./test
rm test