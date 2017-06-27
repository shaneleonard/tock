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
use port_signpost_tock;


pub struct SignbusIOInterface<'a> {
	port_signpost_tock: 	&'a port_signpost_tock::PortSignpostTock<'a>,
	this_device_address:	Cell<u8>,
}

impl<'a> SignbusIOInterface<'a,> {
	pub fn new(port_signpost_tock: &'a port_signpost_tock::PortSignpostTock<'a>
	
	) -> SignbusIOInterface <'a> {
		
		SignbusIOInterface {
			port_signpost_tock:  		port_signpost_tock,
			this_device_address:		Cell::new(0),	
		}
	}

	pub fn signbus_io_init(&self, address: u8) {
		self.this_device_address.set(address);
		self.port_signpost_tock.init(address);
			
		debug!("Address: {}", self.this_device_address.get());
	}
	
}
