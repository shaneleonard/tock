/// Kernel implementation of port_signpost_tock 
/// apps/libsignpost/port_signpost_tock.c -> kernel/tock/capsules/src/port_signpost_tock.rs
/// By: Justin Hsieh

use core::cell::Cell;
use core::cmp;
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil;
use kernel::hil::gpio;
use kernel::hil::i2c;
use kernel::hil::time;
use kernel::ReturnCode

// Buffers to use for I2C messages
pub static mut BUFFER0: [u8; 256] = [0; 256];
pub static mut BUFFER1: [u8; 256] = [0; 256];
pub static mut BUFFER2: [u8; 256] = [0; 256];
pub static mut BUFFER3: [u8; 256] = [0; 256];

#[derive(Clone,Copy,PartialEq)]
enum MasterAction {
	Read(u8),
	Write,
}

pub struct I2CMasterSlaveDriver<'a> {
	i2c: &'a hil::i2c::I2CMasterSlave,
	listening:			Cell<bool>,	
	master_action: 		Cell<MasterAction>,
	master_buffer:		TakeCell <'static, [u8]>,
	slave_buffer1:		TakeCell <'static, [u8]>,
	slave_buffer2:		TakeCell <'static, [u8]>,
}



enum initialization_state {
	Start = 0,
	Isolated = 1,
	KeyExchange = 2,
	Done = 3,
} initialization_state_t;

enum initialization_message_type {
	InitializationDeclare = 0,
	InitializationKeyExchange = 1,
	InitializationGetMods = 2,
}

enum module_address {
	ModuleAddressController = 0x20,
	ModuleAddressStorage = 0x21,
	ModuleAddressRadio = 0x22,
} module_address_t;


/// States of the I2C protocol for Signbus
#[derive(Clone,Copy,PartialEq)]
enum State {
	Idle,
	Init,
}

pub struct PortSignpostTock<'a, A: time::Alarm + 'a> {
	i2c: 		&'a hil::i2c::I2CMasterSlave,
	alarm:		&'a A,

	master_write_yield_flag: mut bool,
	master_write_len_or_rc: mut i32,

	callback:	Cell<Option<Callback>>,
	state:		Cell<State>,

	master_write_buf:	TakeCell <'static, [u8]>,
	slave_read_buf:		TakeCell <'static, [u8]>,
}

impl<'a, A: time::Alarm + 'a> PortSignpostTock<'a, A> {
	pub fn new(	i2c: &'a I2CMasterSlave, 
				alarm: &'a A
				) -> PortSignpostTock<'a, A> {

		PortSignpostTock {
			i2c:  		i2c,
			alarm: 		alarm,

			master_write_yield_flag: 	Cell::new(false),
			master_write_len_or_rc: 	Cell::new(0),
			
			callback: 	Cell::new(None),
			state:		Cell::new(State::Idle),

			master_write_buf:		TakeCell::new(master_write_buf),
			slave_read_buf:			TakeCell::new(slave_read_buf),

		}
	}
/*
	fn set_master_write_buffer(&self) -> ReturnCode {
		self.i2c.app.master_tx_buffer = BUFFER0;	
		return ReturnCode::SUCCESS;
	}
	
	fn set_slave_read_buffer(&self) -> ReturnCode {
		self.i2c.app.slave_rx_buffer = BUFFER1;		
		return ReturnCode::SUCCESS;
	}
*/	
	fn set_slave_address(&self, i2c_address as u8) -> ReturnCode {
		if i2c_address > 0x7f {
			return ReturnCode::EINVAL;
		}
		hil::i2c::I2CSlave::set_address(self.i2c, i2c_address);
		return ReturnCode::SUCCESS;
	}
/*
	fn set_callback(&self, callback: Callback) -> ReturnCode {
		self.i2c.app.callback = callback;
		return ReturnCode::SUCCESS;	
	}
*/
	
	pub fn init(&self, i2c_address as u8) {
		
		let r = set_slave_address(i2c_address);
		if r == ReturnCode::SUCCESS {
			self.state.set(State::Init);
		}

	}

	pub fn i2c_master_write(&self, address as u8, len as u32) {
		self.master_write_yield_flag = false;

		self.
	
		
	}

	pub fn i2c_slave_listen() {
		

	}

	pub fn i2c_slave_read_setup() {

	}

}


impl<'a, A: time::Alarm + 'a> i2c::I2CClient for PortSignpostTock<'a, A> {
	// Link from I2C capsule to PortSignpostTock capsule
	// fn command_complete ()
}

impl<'a, A: time::Alarm + 'a> time::Client for PortSignpostTock<'a, A> [
	// Link from time capsule to PortSignpostTock capsule
	// fn fired ()
}




