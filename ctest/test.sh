cargo build --release
echo "--------------------------"
echo "Test test_raw_allocator..."
gcc -g -o test test_raw_allocator.c -I ../csrc -L../target/release -Wl,-Bstatic -ldpi_memory  -Wl,-Bdynamic -lpthread -ldl -lm
#gcc -g test_raw_allocator.c -ldpi_memory -L../target/release -Wl,-rpath=../target/release -I ../csrc -o test
./test
rm test
echo "Test test_raw_allocator Done!"
echo "--------------------------"
echo "--------------------------"
echo "Test test_region..."
gcc -g -o test test_region.c -I ../csrc -L../target/release -Wl,-Bstatic -ldpi_memory  -Wl,-Bdynamic -lpthread -ldl -lm

./test
rm test
echo "Test test_region Done!"
echo "--------------------------"