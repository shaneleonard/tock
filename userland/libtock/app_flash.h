#pragma once

// Utility for allowing apps to read and write a region of flash. This interface
// is designed for apps to specify a special region of flash that stores
// persistent data, and not as a general purpose flash reading and writing tool.
//
// The expected use looks something like:
//
//   struct valuable_data {
//     uint32_t magic;
//     uint32_t my_value1;
//     uint32_t my_value2;
//   }
//
//   // Using const will tell the compiler to keep this in flash.
//   const struct valuable_data data_in_flash;
//
//   // Also need a copy in RAM that can be used as a scratchpad.
//   struct valuable_data data_in_ram;
//
//   int main() {
//     int ret;
//
//     // Setup the flash region and the shadow copy in RAM.
//     ret = app_flash_configure(&data_in_flash, &data_in_ram, sizeof(data_in_ram));
//     if (ret != 0) prinrf("ERROR(%i): Could not configure the flash region.\n", ret);
//
//     // Get the initial copy of the data in flash.
//     ret = app_flash_read();
//     if (ret != 0) prinrf("ERROR(%i): Could not read the flash region.\n", ret);
//
//     // Check that the magic value is as expected.
//     if (data_in_ram.magic != MAGIC) {
//       // do some initialization
//     }
//
//     // do other computation
//
//     // Save changed valuable data.
//     ret = app_flash_write_sync();
//     if (ret != 0) prinrf("ERROR(%i): Could not write back to flash.\n", ret);
//   }

#ifdef __cplusplus
extern "C" {
#endif

#include "tock.h"

#define DRIVER_NUM_APP_FLASH 30

// Provide a pointer to the start of the region of the app's flash that it
// wishes to access and a buffer in memory that will act as temporary storage
// for the data actually stored in flash. This buffer will be populated with the
// contents of flash when a read() is called, and will be written to flash when
// write() is called. This buffer must be in RAM and must not point directly to
// the flash address space.
int app_flash_configure(void* flash, void* buffer, uint32_t length);

// Set a callback function for when the flash has been written to.
int app_flash_set_callback(subscribe_cb callback, void* callback_args);

// Populate the provided buffer (in the configure call) with the data stored
// in flash at the address also provided to configure(). The entire buffer will
// be populated.
int app_flash_read_sync(void);

// Write the contents of the buffer in RAM to flash at the address specified
// in the configure() call.
int app_flash_write(void);
int app_flash_write_sync(void);

#ifdef __cplusplus
}
#endif
