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

/// States of the I2C protocol for Signbus
#[derive(Clone,Copy,PartialEq)]
enum State {
	Idle,
	Init,
	MasterWrite,
	MasterRead,
	SlaveWrite,
	SlaveRead,
}

pub struct PortSignpostTock<'a, A: time::Alarm + 'a> {
	i2c: 		&'a hil::i2c::I2CMasterSlave,
	alarm:		&'a A,
	
	listening:	Cell<bool>,
	master_action: 		Cell<MasterAction>,
	
	master_tx_buffer:		TakeCell <'static, [u8]>,
	master_rx_buffer:		TakeCell <'static, [u8]>,
	slave_tx_buffer:		TakeCell <'static, [u8]>,
	slave_rx_buffer:		TakeCell <'static, [u8]>,

	state:		Cell<State>,
}

impl<'a, A: time::Alarm + 'a> PortSignpostTock<'a, A> {
	pub fn new(	i2c: &'a I2CMasterSlave, alarm: &'a A) -> PortSignpostTock<'a, A> {
		PortSignpostTock {
			i2c:  		i2c,
			alarm: 		alarm,

			listening:				
			master_action:
			
			master_tx_buffer:		TakeCell::new(master_tx_buffer),
			master_rx_buffer:		TakeCell::new(master_rx_buffer),
			slave_tx_buffer:		TakeCell::new(slave_tx_buffer),
			slave_rx_buffer:		TakeCell::new(slave_rx_buffer),
			
			state:		Cell::new(State::Idle),
		}
	}
	
	fn set_slave_address(&self, i2c_address as u8) -> ReturnCode {

		if i2c_address > 0x7f {
			ReturnCode::EINVAL;
		}
		hil::i2c::I2CSlave::set_address(self.i2c, i2c_address);

		ReturnCode::SUCCESS;
	}
	
	pub fn init(&self, i2c_address as u8) -> ReturnCode {
		
		let r = set_slave_address(i2c_address);
		if r == ReturnCode::SUCCESS {
			self.state.set(State::Init);
		}

		return r;
	}

	pub fn i2c_master_write(&self, address as u8, len as u32) -> ReturnCode {
	
		self.master_tx_buffer.as_mut().map(|buffer|{
		
			hil::i2c::I2CMaster::enable(self.i2c);
			hil::i2c::I2CMaster::write(self.i2c, address, buffer, len as u8);
		});
		
		// TODO: yield() or implement client callback

		ReturnCode::SUCCESS;
	}

	pub fn i2c_slave_listen(&self) -> ReturnCode {

		self.slave_rx_buffer.take().map(|buffer| {
			hil::i2c::I2CSlave::write_receive(self.i2c, buffer, 255);
		});

		hil::i2c::I2CSlave::enable(self.i2c);
		hil::i2c::I2CSlave::listen(self.i2c);


		self.state.set(State::SlaveRead);
		ReturnCode::SUCCESS;	
	}

	pub fn i2c_slave_read_setup(&self, len as u32) -> ReturnCode {
		self.slave_tx_buffer.as_mut().map(|buffer| {
			hil::i2c::I2CSlave::read_send(self.i2c, buffer, len as u8);
		});

		self.state.set(State::MasterRead);
		ReturnCode::SUCCESS;	
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




