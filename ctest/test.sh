cargo build --release
echo "--------------------------"
echo "Test test_raw_allocator..."
#gcc -g -o test test_raw_allocator.c -I ../csrc -L../target/release -Wl,-Bstatic -lterminus_spaceport  -Wl,-Bdynamic -lpthread -ldl -lm
gcc -g test_raw_allocator.c -lts.c -lterminus_spaceport -L../target/release -Wl,-rpath=../target/release -I ../target/release -o test
./test
rm test
echo "Test test_raw_allocator Done!"
echo "--------------------------"
echo "--------------------------"
echo "Test test_region..."
gcc -g -o test test_region.c -I ../target/release -L../target/release -Wl,-Bstatic -lterminus_spaceport  -Wl,-Bdynamic -lpthread -ldl -lm -lrt

./test
rm test
echo "Test test_region Done!"
echo "--------------------------"