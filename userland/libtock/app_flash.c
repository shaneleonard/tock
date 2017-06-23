#include <string.h>

#include "app_flash.h"
#include "tock.h"

// Some state that this library needs to implement all of the functions.
void* _flash_pointer        = NULL;
void* _ram_buffer           = NULL;
uint32_t _ram_buffer_length = 0;

// For synchronous calls.
struct app_flash_data {
  bool fired;
};

static struct app_flash_data result = { .fired = false };

// Internal callback for faking synchronous reads
static void app_flash_cb(__attribute__ ((unused)) int callback_type,
                         __attribute__ ((unused)) int value,
                         __attribute__ ((unused)) int unused,
                         void* ud) {
  struct app_flash_data* myresult = (struct app_flash_data*) ud;
  myresult->fired = true;
}


int app_flash_configure(void* flash, void* buffer, uint32_t length) {
  _flash_pointer     = flash;
  _ram_buffer        = buffer;
  _ram_buffer_length = length;

  return allow(DRIVER_NUM_APP_FLASH, 0, buffer, length);
}

int app_flash_set_callback(subscribe_cb callback, void* callback_args) {
  return subscribe(DRIVER_NUM_APP_FLASH, 0, callback, callback_args);
}

int app_flash_read_sync(void) {
  memcpy((uint8_t*) _ram_buffer, (uint8_t*) _flash_pointer, _ram_buffer_length);
  return 0;
}

int app_flash_write(void) {
  return command(DRIVER_NUM_APP_FLASH, 1, (uint32_t) _flash_pointer);
}



int app_flash_write_sync(void) {
  int err;
  result.fired = false;

  err = app_flash_set_callback(app_flash_cb, (void*) &result);
  if (err < 0) return err;

  err = app_flash_write();
  if (err < 0) return err;

  // Wait for the callback.
  yield_for(&result.fired);

  return 0;
}
