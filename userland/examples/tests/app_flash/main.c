#include <stdio.h>

#include <app_flash.h>
#include <tock.h>

const uint8_t my_saved_buffer[100] = {9};

uint8_t ram_buffer[100];


int main(void) {
  int ret;

  printf("My flash buffer address: %p\n", my_saved_buffer);

  ret = app_flash_configure(my_saved_buffer, ram_buffer, 100);
  if (ret != 0) printf("ERROR configuring flash: %i\n", ret);

  app_flash_read_sync();
  printf("Read bytes:\n");
  for (int i = 0; i < 65; i++) {
    printf("0x%x ", ram_buffer[i]);
  }
  printf("\n");

  printf("Setting new bytes.\n");
  uint8_t start = ram_buffer[0];
  for (int i = 0; i < 65; i++) {
    ram_buffer[i] = start + i + 10;
  }
  ret = app_flash_write();
  if (ret != 0) printf("ERROR writing flash: %i\n", ret);

  printf("Reset board to see updated flash.\n");

  return 0;
}
