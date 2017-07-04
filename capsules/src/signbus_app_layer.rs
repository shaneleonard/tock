/// Kernel implementation of signbus_app_layer
/// apps/libsignpost/signbus_app_layer.c -> kernel/tock/capsules/src/signbus_app_layer.rs
/// By: Justin Hsieh

use core::cell::Cell;
use core::cmp;
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil;
use kernel::hil::gpio;
use kernel::hil::time;
// Capsules
use signbus_protocol_layer;

pub static mut BUFFER0: [u8; 256] = [0; 256];
pub static mut BUFFER1: [u8; 256] = [0; 256];
pub static mut BUFFER2: [u8; 256] = [1; 256];


pub struct App {
	callback: Option<Callback>,
	master_tx_buffer: Option<AppSlice<Shared, u8>>,
	master_rx_buffer: Option<AppSlice<Shared, u8>>,
	slave_tx_buffer: Option<AppSlice<Shared, u8>>,
	slave_rx_buffer: Option<AppSlice<Shared, u8>>,
}

impl Default for App {
	fn default() -> App {
		App {
			callback: None,
			master_tx_buffer: None,
			master_rx_buffer: None,
			slave_tx_buffer: None,
			slave_rx_buffer: None,
		}
	}
}

pub struct SignbusAppLayer<'a> {
	signbus_protocol_layer: 	&'a signbus_protocol_layer::SignbusProtocolLayer<'a>,
}

impl<'a> SignbusAppLayer<'a,> {
	pub fn new(signbus_protocol_layer: &'a signbus_protocol_layer::SignbusProtocolLayer<'a>
	
	) -> SignbusAppLayer <'a> {
		
		SignbusAppLayer {
			signbus_protocol_layer:  		signbus_protocol_layer,
		}
	}

	pub fn signbus_app_send(&self, address: u8) {
		//self.signbus_protocol_layer.signbus_protocol_send(address);
	}
	
}
