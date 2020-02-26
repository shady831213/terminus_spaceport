#include "verilated.h"
#include <iostream>
#include <fstream>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/stat.h>
#include <VTestModule.h>
extern "C" {
    #include <wrap.h>
}
static uint64_t trace_count = 0;

static void* root_space = dm_new_space();

double sc_time_stamp()
{
  return trace_count;
}

extern "C" void* dm_get_space(char* name) {
    return root_space;
}

int main(int argc, char** argv)
{
  unsigned random_seed = (unsigned)time(NULL) ^ (unsigned)getpid();
  uint64_t max_cycles = -1;
  int ret = 0;
  srand(random_seed);
  srand48(random_seed);

  Verilated::randReset(2);
  Verilated::commandArgs(argc, argv);

  void* main_memory = dm_alloc_region(NULL,16ul, 8ul);
  void* rom = dm_alloc_region(NULL, 8ul, 8ul);

  dm_add_region(root_space, "main_memory", dm_map_region(main_memory, 0x800000000ul));
  dm_add_region(root_space, "rom",  dm_map_region(rom, 0x100000000ul));

  dm_region_write_u32(rom, dm_c_region_info(rom)->base, 0x5a5aa5a5u);


  VTestModule *top = new VTestModule;
  top->clock = 0;
  while (!Verilated::gotFinish()) {
    if (trace_count%2 == 1) {
        top->clock = 1;
    } else {
        top->clock = 0;
    }
    top->eval();
    trace_count++;
  }
  std::cout << std::hex << "read main_memory @" << dm_c_region_info(main_memory)->base << "data=" << dm_c_region_read_u16(main_memory, dm_c_region_info(main_memory)->base) << std::endl;
  std::cout << std::hex << "read main_memory @" << dm_c_region_info(main_memory)->base+2 << "data=" << dm_c_region_read_u16(main_memory, dm_c_region_info(main_memory)->base+2) << std::endl;

  std::cout << "Done!" << std::endl;

  dm_free_region(main_memory);
  dm_free_region(rom);
  dm_delete_region(root_space, "main_memory");
  dm_delete_region(root_space, "rom");

  return ret;
}