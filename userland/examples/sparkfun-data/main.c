#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <ble_advdata.h>
#include <nordic_common.h>
#include <nrf_error.h>

#include <eddystone.h>
#include <simple_adv.h>
#include <simple_ble.h>

#include <nrf51_serialization.h>

#include <console.h>
#include <led.h>
#include <tock.h>

#include "nrf.h"


/*******************************************************************************
 * BLE
 ******************************************************************************/

uint16_t conn_handle = BLE_CONN_HANDLE_INVALID;

// Intervals for advertising and connections
simple_ble_config_t ble_config = {
  .platform_id       = 0x00,                // used as 4th octect in device BLE address
  .device_id         = DEVICE_ID_DEFAULT,
  .adv_name          = "tock-http",
  .adv_interval      = MSEC_TO_UNITS(500, UNIT_0_625_MS),
  .min_conn_interval = MSEC_TO_UNITS(1000, UNIT_1_25_MS),
  .max_conn_interval = MSEC_TO_UNITS(1250, UNIT_1_25_MS)
};



__attribute__ ((const))
void ble_address_set (void) {
  // nop
}

uint16_t _char_handle = 0;
uint16_t _char_decl_handle = 0;

char get[] = "GET https://j2x.us/\r\nhost: j2x.us\r\n\r\n";


int _write_offset = 0;
int _write_state = 0;

void write_http_string_loop () {
  uint32_t err_code;
  if (_write_state == 2) {
    // done!
    return;
  }

  ble_gattc_write_params_t write_params;
  memset(&write_params, 0, sizeof(write_params));

  if (_write_state == 0) {

    int len = sizeof(get) - 1 - _write_offset;
    if (len > 18) {
      len = 18;
    }



    write_params.handle     = _char_handle;
    write_params.write_op   = BLE_GATT_OP_PREP_WRITE_REQ;
    // write_params.write_op   = BLE_GATT_OP_WRITE_REQ;
    // write_params.write_op   = BLE_GATT_OP_WRITE_CMD;
    // write_params.flags      = BLE_GATT_EXEC_WRITE_FLAG_PREPARED_WRITE;
    write_params.offset     = _write_offset;
    write_params.len        = len;
    write_params.p_value    = get + _write_offset;



    _write_offset += len;

    if (_write_offset >= sizeof(get) - 1) {
      _write_state = 1;
    }

  } else if (_write_state == 1) {
    write_params.handle     = _char_handle;
    write_params.write_op   = BLE_GATT_OP_EXEC_WRITE_REQ;
    write_params.flags      = BLE_GATT_EXEC_WRITE_FLAG_PREPARED_WRITE;
    // write_params.offset     = _write_offset;
    // write_params.len        = len;
    // write_params.p_value    = get + _write_offset;
    _write_state = 2;
  } else if (_write_state == 3) {
    // try to enable notifications
    uint8_t buf[2];
    buf[0] = BLE_GATT_HVX_NOTIFICATION;
    buf[1] = 0;

    write_params.write_op = BLE_GATT_OP_WRITE_REQ;
    write_params.handle   = _char_decl_handle;
    write_params.offset   = 0;
    write_params.len      = sizeof(buf);
    write_params.p_value  = buf;

    _write_state = 0;
  }

  // printf("len: %i, p: %p, char: 0x%x\n", write_params.len, write_params.p_value, write_params.handle);

  printf("write quick get string %i %i\n", _write_offset, _write_state);
  err_code = sd_ble_gattc_write(conn_handle, &write_params);
  if (err_code != NRF_SUCCESS) {
    printf("error writing Characteristic 0x%x\n", err_code);
  }
}

void write_http_string () {
  _write_offset = 0;
  _write_state = 3;
  write_http_string_loop();
}


void ble_evt_user_handler (ble_evt_t* p_ble_evt) {
  uint32_t err_code;


  switch (p_ble_evt->header.evt_id) {
    case BLE_GAP_EVT_CONN_PARAM_UPDATE: {
      // just update them right now
      ble_gap_conn_params_t conn_params;
      memset(&conn_params, 0, sizeof(conn_params));
      conn_params.min_conn_interval = ble_config.min_conn_interval;
      conn_params.max_conn_interval = ble_config.max_conn_interval;
      conn_params.slave_latency     = SLAVE_LATENCY;
      conn_params.conn_sup_timeout  = CONN_SUP_TIMEOUT;

      sd_ble_gap_conn_param_update(0, &conn_params);
      break;
    }

    case BLE_GAP_EVT_ADV_REPORT: {
      // ignore
      break;
    }

    case BLE_EVT_TX_COMPLETE: {
      printf("tx complete\n");
      break;
    }

    case BLE_GATTC_EVT_PRIM_SRVC_DISC_RSP: {
      if (p_ble_evt->evt.gattc_evt.gatt_status != BLE_GATT_STATUS_SUCCESS ||
          p_ble_evt->evt.gattc_evt.params.prim_srvc_disc_rsp.count == 0) {
          printf("Service not found\n");
      } else {
        // There should be only one instance of the service at the peer.
        // So only the first element of the array is of interest.
        const ble_gattc_handle_range_t* p_service_handle_range = &(p_ble_evt->evt.gattc_evt.params.prim_srvc_disc_rsp.services[0].handle_range);

        // Discover characteristics.
        err_code = sd_ble_gattc_characteristics_discover(conn_handle, p_service_handle_range);
        if (err_code != NRF_SUCCESS) {
          printf("error discover char 0x%x\n", err_code);
        }
      }
      break;
    }

    case BLE_GATTC_EVT_CHAR_DISC_RSP: {
      if (p_ble_evt->evt.gattc_evt.gatt_status != BLE_GATT_STATUS_SUCCESS) {
        printf("Characteristic not found\n");
      } else {


        int i;
        const ble_gattc_evt_char_disc_rsp_t* p_char_disc_rsp;

        p_char_disc_rsp = &(p_ble_evt->evt.gattc_evt.params.char_disc_rsp);

        ble_uuid_t httpget_characteristic_uuid = {
          .uuid = 0x0002,
          .type = BLE_UUID_TYPE_VENDOR_BEGIN,
        };

        // Iterate through the characteristics and find the correct one.
        for (i = 0; i < p_char_disc_rsp->count; i++) {
          printf("char: uuid: 0x%x type: 0x%x\n", p_char_disc_rsp->chars[i].uuid.uuid, p_char_disc_rsp->chars[i].uuid.type);
          if (BLE_UUID_EQ(&httpget_characteristic_uuid, &(p_char_disc_rsp->chars[i].uuid))) {
            printf("found char handle 0x%x\n", p_char_disc_rsp->chars[i].handle_decl);
            _char_handle = p_char_disc_rsp->chars[i].handle_value;
            _char_decl_handle = p_char_disc_rsp->chars[i].handle_decl;
            // prinp_char_disc_rsp->chars[i].handle_decl;



            _write_offset = 0;
            write_http_string();


            break;
          }
        }
      }
      break;
    }

    case BLE_GATTC_EVT_WRITE_RSP: {
      write_http_string_loop();
      break;
    }

    default:
      printf("event 0x%x\n", p_ble_evt->header.evt_id);
  }
}

// // This gets called with the serial data from the BLE central.
// static void nus_data_handler(ble_nus_t* p_nus, uint8_t* p_data, uint16_t length) {
//   UNUSED_PARAMETER(p_nus);

//   // In this app, just print it to the console.
//   putnstr((char*) p_data, length);
// }



void ble_evt_connected(ble_evt_t* p_ble_evt) {
  // ble_common_evt_t *common = (ble_common_evt_t*) &p_ble_evt->evt;
  // conn_handle = common->conn_handle;

  conn_handle = p_ble_evt->evt.gap_evt.conn_handle;

  // ble_nus_on_ble_evt(&m_nus, p_ble_evt);

  ble_uuid_t httpget_service_uuid = {
    .uuid = 0x0001,
    .type = BLE_UUID_TYPE_VENDOR_BEGIN,
  };

  printf("discover services %x\n", conn_handle);
  uint32_t err_code = sd_ble_gattc_primary_services_discover(conn_handle, 0x0001, &httpget_service_uuid);
  if (err_code != NRF_SUCCESS) {
    printf("error discovering services 0x%x\n", err_code);
  }
}

void ble_evt_disconnected(ble_evt_t* p_ble_evt) {
  conn_handle = BLE_CONN_HANDLE_INVALID;
  printf("disconn\n");

  // ble_nus_on_ble_evt(&m_nus, p_ble_evt);
}

// // On a write, need to forward that to NUS library.
// void ble_evt_write(ble_evt_t* p_ble_evt) {
//   ble_nus_on_ble_evt(&m_nus, p_ble_evt);
// }

void ble_error (uint32_t error_code) {
  printf("BLE ERROR: Code = 0x%x\n", (int)error_code);
}

#define UUID16_SIZE             2                               /**< Size of 16 bit UUID */
#define UUID32_SIZE             4                               /**< Size of 32 bit UUID */
#define UUID128_SIZE            16                              /**< Size of 128 bit UUID */



#define BLEHTTP_BASE_UUID {{0x30, 0xb3, 0xf6, 0x90, 0x9a, 0x4f, 0x89, 0xb8, 0x1e, 0x46, 0x44, 0xcf, 0x01, 0x00, 0xba, 0x16}}
#define BLEHTTP_BASE_UUID_B {0x30, 0xb3, 0xf6, 0x90, 0x9a, 0x4f, 0x89, 0xb8, 0x1e, 0x46, 0x44, 0xcf, 0x01, 0x00, 0xba, 0x16}
#define BLE_UUID_BLEHTTP_SERVICE 0x0001
#define BLE_UUID_BLEHTTP_CHAR 0x0002

#define MIN_CONNECTION_INTERVAL MSEC_TO_UNITS(20, UNIT_1_25_MS) /**< Determines minimum connection interval in millisecond. */
#define MAX_CONNECTION_INTERVAL MSEC_TO_UNITS(75, UNIT_1_25_MS) /**< Determines maximum connection interval in millisecond. */
#define SLAVE_LATENCY           0                               /**< Determines slave latency in counts of connection events. */
#define SUPERVISION_TIMEOUT     MSEC_TO_UNITS(4000, UNIT_10_MS) /**< Determines supervision time-out in units of 10 millisecond. */


static const ble_uuid_t m_blehttp_uuid = {
  .uuid = BLE_UUID_BLEHTTP_SERVICE,
  .type = BLE_UUID_TYPE_VENDOR_BEGIN
};

static const ble_gap_conn_params_t m_connection_param = {
  (uint16_t)MIN_CONNECTION_INTERVAL,  // Minimum connection
  (uint16_t)MAX_CONNECTION_INTERVAL,  // Maximum connection
  (uint16_t)SLAVE_LATENCY,            // Slave latency
  (uint16_t)SUPERVISION_TIMEOUT       // Supervision time-out
};

static const ble_gap_scan_params_t m_scan_param = {
  .active = 0,                   // Active scanning not set.
  .selective = 0,                // Selective scanning not set.
  .p_whitelist = NULL,           // No whitelist provided.
  .interval = 0x00A0,
  .window = 0x0050,
  .timeout = 0x0000
};

static uint8_t blehttp_service_id[16] = BLEHTTP_BASE_UUID_B;



static bool is_uuid_present(const ble_uuid_t *p_target_uuid,
                            const ble_gap_evt_adv_report_t *p_adv_report) {
  uint32_t err_code;
  uint32_t index = 0;
  uint8_t *p_data = (uint8_t *)p_adv_report->data;
  ble_uuid_t extracted_uuid;

  while (index < p_adv_report->dlen) {
    uint8_t field_length = p_data[index];
    uint8_t field_type   = p_data[index+1];

    // if ((field_type == BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_MORE_AVAILABLE)
    //     || (field_type == BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_COMPLETE)) {
    //   for (uint32_t u_index = 0; u_index < (field_length/UUID16_SIZE); u_index++) {
    // // printf("[0\n");
    //     err_code = sd_ble_uuid_decode(UUID16_SIZE,
    //                                   &p_data[u_index * UUID16_SIZE + index + 2],
    //                                   &extracted_uuid);
    // // printf("[1\n");
    //     if (err_code == NRF_SUCCESS) {
    //       if ((extracted_uuid.uuid == p_target_uuid->uuid)
    //           && (extracted_uuid.type == p_target_uuid->type)) {
    //         return true;
    //       }
    //     }
    //   }
    // } else if ((field_type == BLE_GAP_AD_TYPE_32BIT_SERVICE_UUID_MORE_AVAILABLE)
    //            || (field_type == BLE_GAP_AD_TYPE_32BIT_SERVICE_UUID_COMPLETE)) {
    //   for (uint32_t u_index = 0; u_index < (field_length/UUID32_SIZE); u_index++) {
    //     err_code = sd_ble_uuid_decode(UUID16_SIZE,
    //     &p_data[u_index * UUID32_SIZE + index + 2],
    //     &extracted_uuid);
    //     if (err_code == NRF_SUCCESS) {
    //       if ((extracted_uuid.uuid == p_target_uuid->uuid)
    //           && (extracted_uuid.type == p_target_uuid->type)) {
    //         return true;
    //       }
    //     }
    //   }
    // } else
    if ((field_type == BLE_GAP_AD_TYPE_128BIT_SERVICE_UUID_MORE_AVAILABLE)
        || (field_type == BLE_GAP_AD_TYPE_128BIT_SERVICE_UUID_COMPLETE)) {
    // printf("[0\n");

      // if
      if (field_length - 1 == 16 &&
          memcmp(blehttp_service_id, &p_data[index + 2], 16) == 0) {
        printf("hey found\n");

        printf("Connecting to target %02x%02x%02x%02x%02x%02x\n",
                         p_adv_report->peer_addr.addr[0],
                         p_adv_report->peer_addr.addr[1],
                         p_adv_report->peer_addr.addr[2],
                         p_adv_report->peer_addr.addr[3],
                         p_adv_report->peer_addr.addr[4],
                         p_adv_report->peer_addr.addr[5]
                         );

        return true;
      }

  //     err_code = sd_ble_uuid_decode(UUID128_SIZE,
  //                                   &p_data[index + 2],
  //                                   &extracted_uuid);
  // // printf("[1\n");
  //     if (err_code == NRF_SUCCESS) {
  //       if ((extracted_uuid.uuid == p_target_uuid->uuid)
  //           && (extracted_uuid.type == p_target_uuid->type)) {
  //         return true;
  //       }
  //     }
    }
    index += field_length + 1;
  }
  return false;
}

static void extract_name(const ble_gap_evt_adv_report_t *p_adv_report, char* buffer) {
  uint32_t err_code;
  uint32_t index = 0;
  uint8_t *p_data = (uint8_t *)p_adv_report->data;
  ble_uuid_t extracted_uuid;

  while (index < p_adv_report->dlen) {
    uint8_t field_length = p_data[index];
    uint8_t field_type   = p_data[index+1];

    if ((field_type == BLE_GAP_AD_TYPE_SHORT_LOCAL_NAME)
        || (field_type == BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME)) {
      memcpy(buffer, &p_data[index+2], field_length-1);
    }
    index += field_length + 1;
  }
}

void blehttp_init (void) {
  ble_uuid128_t base_uuid = BLEHTTP_BASE_UUID;
  uint8_t base_uuid_type = BLE_UUID_TYPE_VENDOR_BEGIN;

  sd_ble_uuid_vs_add(&base_uuid, &base_uuid_type);
}



void ble_evt_adv_report (ble_evt_t* p_ble_evt) {
  uint32_t err_code;
  led_toggle(2);


  const ble_gap_evt_t* p_gap_evt = &p_ble_evt->evt.gap_evt;
  const ble_gap_evt_adv_report_t* p_adv_report = &p_gap_evt->params.adv_report;

  if (is_uuid_present(&m_blehttp_uuid, p_adv_report)) {
    // gpio_toggle(1);
    printf("yay\n");
    char device_name[31] = {'?'};
    extract_name(p_adv_report, device_name);
    printf("found %s\n", device_name);

    err_code = sd_ble_gap_connect(&p_adv_report->peer_addr,
                                  &m_scan_param,
                                  &m_connection_param);

    if (err_code == NRF_SUCCESS) {
      printf("called connect.\n");
        // // scan is automatically stopped by the connect
        // err_code = bsp_indication_set(BSP_INDICATE_IDLE);
        // APP_ERROR_CHECK(err_code);
        // APPL_LOG("Connecting to target %02x%02x%02x%02x%02x%02x\r\n",
        //          p_adv_report->peer_addr.addr[0],
        //          p_adv_report->peer_addr.addr[1],
        //          p_adv_report->peer_addr.addr[2],
        //          p_adv_report->peer_addr.addr[3],
        //          p_adv_report->peer_addr.addr[4],
        //          p_adv_report->peer_addr.addr[5]
        //          );




      // const ble_gattc_write_params_t write_params = {
      //     .write_op = BLE_GATT_OP_WRITE_CMD,
      //     .flags    = BLE_GATT_EXEC_WRITE_FLAG_PREPARED_WRITE,
      //     .handle   = p_ble_nus_c->handles.nus_tx_handle,
      //     .offset   = 0,
      //     .len      = sizeof(get),
      //     .p_value  = get
      // };

      // err_code = sd_ble_gattc_write(conn_handle, &write_params);
      // if (err_code == NRF_SUCCESS) {
      //   printf("wrote\n");
      // } else {
      //   printf("error writing %i\n", err_code);
      // }
    }
  }
}

// void services_init (void) {
//   uint32_t err_code;
//   ble_nus_init_t nus_init;
//   memset(&nus_init, 0, sizeof(nus_init));
//   nus_init.data_handler = nus_data_handler;
//   err_code = ble_nus_init(&m_nus, &nus_init);
//   APP_ERROR_CHECK(err_code);
// }





/*******************************************************************************
 * MAIN
 ******************************************************************************/

int main (void) {
  uint32_t err_code;

  printf("[BLE] Find Gateway\n");

  // // Setup BLE
  // conn_handle = simple_ble_init(&ble_config)->conn_handle;

  // // Advertise the UART service
  // ble_uuid_t adv_uuid = {0x0001, BLE_UUID_TYPE_VENDOR_BEGIN};
  // simple_adv_service(&adv_uuid);

  gpio_enable_output(0);
  gpio_enable_output(1);
  gpio_enable_output(2);


  simple_ble_init(&ble_config);

  blehttp_init();




  err_code = sd_ble_gap_scan_stop();

  err_code = sd_ble_gap_scan_start(&m_scan_param);
  // It is okay to ignore this error since we are stopping the scan anyway.
  if (err_code != NRF_ERROR_INVALID_STATE) {
    APP_ERROR_CHECK(err_code);
  }

  while(1) {
    // printf("a\n");
    yield();
  }
}
