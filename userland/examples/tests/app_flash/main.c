#include <stdio.h>

#include <app_flash.h>
#include <tock.h>

const uint8_t my_saved_buffer[1024] = {9};

uint8_t ram_buffer[512];


int main(void) {
  int ret;

  // We need a pointer in flash that is on a 512 byte boundary.
  uint8_t* aligned_buffer = (uint8_t*) (((uint32_t) (my_saved_buffer + 511)) & 0xFFFFFE00);

  printf("My flash buffer address: %p\n", my_saved_buffer);
  printf("My flash buffer address: %p\n", aligned_buffer);

  ret = app_flash_configure(aligned_buffer, ram_buffer, 512);
  if (ret != 0) printf("ERROR configuring flash: %i\n", ret);

  app_flash_read_sync();
  printf("Read bytes:\n");
  for (int i = 0; i < 15; i++) {
    printf("  0x%x\n", ram_buffer[i]);
  }

  printf("Setting new bytes\n");
  uint8_t start = ram_buffer[0];
  for (int i = 0; i < 15; i++) {
    ram_buffer[i] = start + i + 10;
  }
  ret = app_flash_write();
  if (ret != 0) printf("ERROR writing flash: %i\n", ret);

  return 0;
}
