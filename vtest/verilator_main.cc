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
    #include <ts_c.h>
}
static uint64_t trace_count = 0;

static void* space = tsc_space();

extern "C" {
    void* root_space() {
        return space;
    }
}


double sc_time_stamp()
{
  return trace_count;
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

  void* main_memory = tsc_lazy_root_region(16ul, 8ul);
  void* rom = tsc_root_region(8ul, 8ul);

  tsc_add_region(space, "main_memory", tsc_map_region(main_memory, 0x800000000ul));
  tsc_add_region(space, "rom",  tsc_map_region(rom, 0x100000000ul));

  std::cout << std::hex << "write rom @" << tsc_region_info(rom)->base << std::endl;
  tsc_region_write_u32(rom, tsc_region_info(rom)->base, 0x5a5aa5a5);
  std::cout << std::hex << "write rom @" << tsc_region_info(rom)->base <<" Done!"<< std::endl;
  std::cout << std::hex << "read rom @" << tsc_region_info(rom)->base << " data=" << tsc_region_read_u32(rom, tsc_region_info(rom)->base) << std::endl;

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
  std::cout << std::hex << "read main_memory @" << tsc_region_info(main_memory)->base << "data=" << tsc_region_read_u16(main_memory, tsc_region_info(main_memory)->base) << std::endl;
  std::cout << std::hex << "read main_memory @" << tsc_region_info(main_memory)->base+2 << "data=" << tsc_region_read_u16(main_memory, tsc_region_info(main_memory)->base+2) << std::endl;

  std::cout << "Done!" << std::endl;

  tsc_free_region(main_memory);
  tsc_free_region(rom);
  tsc_delete_region(space, "main_memory");
  tsc_delete_region(space, "rom");

  return ret;
}