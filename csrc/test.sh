cargo build
echo "--------------------------"
echo "Test test_raw_allocator..."
gcc -g test_raw_allocator.c wrap.c -ldpi_memory -L../target/debug -Wl,-rpath=../target/debug -I . -o test
#gcc -g -o test test_raw_allocator.c -L../target/debug -Wl,-Bstatic -ldpi_memory  -Wl,-Bdynamic -lpthread -ldl
./test
rm test
echo "Test test_raw_allocator Done!"
echo "--------------------------"
echo "--------------------------"
echo "Test test_region..."
gcc -g test_region.c wrap.c -ldpi_memory -L../target/debug -Wl,-rpath=../target/debug -I . -o test
./test
rm test
echo "Test test_region Done!"
echo "--------------------------"