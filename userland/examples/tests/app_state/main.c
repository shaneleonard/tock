#include <stdio.h>

#include <app_state.h>
#include <tock.h>

#define MAGIC 0xcafe

struct demo_app_state_t {
  uint32_t magic;
  uint32_t count;
};

// n.b. This is equivalent to
//   struct demo_app_state_t app_state;
// but handles a little additional app_state machinery as well.
APP_STATE_DECLARE(struct demo_app_state_t, app_state);

int main(void) {
  int ret;

  // n.b. user code should not access the bare pointers
  printf("DEBUG: Ram addr %p  Flash addr %p\n\n",
      _app_state_ram_pointer, _app_state_flash_pointer);

  ret = app_state_load_sync();
  if (ret < 0) {
    printf("Error loading application state: %s\n", tock_strerror(ret));
    return ret;
  }

  if (app_state.magic != MAGIC) {
    printf("Application has never saved state before\n");
    app_state.magic = MAGIC;
    app_state.count = 1;
  } else {
    printf("This application has run %lu time(s) before\n",
        app_state.count);
    app_state.count += 1;
  }

  ret = app_state_save_sync();
  if (ret != 0) {
    printf("ERROR saving application state: %s\n", tock_strerror(ret));
    return ret;
  }
  printf("State saved successfully. Done.\n");

  return 0;
}
