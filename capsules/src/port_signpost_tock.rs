/// Kernel implementation of port_signpost_tock 
/// apps/libsignpost/port_signpost_tock.c -> kernel/tock/capsules/src/port_signpost_tock.rs
/// By: Justin Hsieh

use core::cell::Cell;
use kernel::{ReturnCode};
use kernel::common::take_cell::{TakeCell};
use kernel::hil;
use kernel::hil::i2c;

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

pub struct PortSignpostTock<'a> {
	i2c: 		&'a hil::i2c::I2CMasterSlave,
	
	pub master_tx_buffer:		TakeCell <'static, [u8]>,
	pub master_rx_buffer:		TakeCell <'static, [u8]>,
	pub slave_tx_buffer:		TakeCell <'static, [u8]>,
	pub slave_rx_buffer:		TakeCell <'static, [u8]>,

	state:		Cell<State>,
}

impl<'a> PortSignpostTock<'a> {
	pub fn new(	i2c: &'a hil::i2c::I2CMasterSlave,
				master_tx_buffer: &'static mut [u8],
				master_rx_buffer: &'static mut [u8],
				slave_tx_buffer: &'static mut [u8],
				slave_rx_buffer: &'static mut [u8]) -> PortSignpostTock<'a> {
		PortSignpostTock {
			i2c:  		i2c,
			master_tx_buffer:		TakeCell::new(master_tx_buffer),
			master_rx_buffer:		TakeCell::new(master_rx_buffer),
			slave_tx_buffer:		TakeCell::new(slave_tx_buffer),
			slave_rx_buffer:		TakeCell::new(slave_rx_buffer),
			state:		Cell::new(State::Idle),
		}
	}
	
	fn set_slave_address(&self, i2c_address: u8) -> ReturnCode {

		if i2c_address > 0x7f {
			return ReturnCode::EINVAL;
		}
		hil::i2c::I2CSlave::set_address(self.i2c, i2c_address);

		return ReturnCode::SUCCESS;
	}
	
	pub fn init(&self, i2c_address: u8) -> ReturnCode {
		
		let r = self.set_slave_address(i2c_address);
		if r == ReturnCode::SUCCESS {
			self.state.set(State::Init);
		}

		return r;
	}

	pub fn i2c_master_write(&self, address: u8, len: u16) -> ReturnCode {
		
		self.master_tx_buffer.take().map(|buffer|{
			hil::i2c::I2CMaster::enable(self.i2c);
			hil::i2c::I2CMaster::write(self.i2c, address, buffer, len as u8);
		});
		
		// TODO: yield() or implement client callback

		return ReturnCode::SUCCESS;
	}

	pub fn i2c_slave_listen(&self) -> ReturnCode {

		self.slave_rx_buffer.take().map(|buffer| {
			hil::i2c::I2CSlave::write_receive(self.i2c, buffer, 255);
		});

		hil::i2c::I2CSlave::enable(self.i2c);
		hil::i2c::I2CSlave::listen(self.i2c);


		self.state.set(State::SlaveRead);
		return ReturnCode::SUCCESS;
	}

	pub fn i2c_slave_read_setup(&self, len: u32) -> ReturnCode {
		self.slave_tx_buffer.take().map(|buffer| {
			hil::i2c::I2CSlave::read_send(self.i2c, buffer, len as u8);
		});

		self.state.set(State::MasterRead);
		return ReturnCode::SUCCESS;	
	}

}


impl<'a> i2c::I2CHwMasterClient for PortSignpostTock <'a> {
	fn command_complete(&self, buffer: &'static mut [u8], error: hil::i2c::Error) {
		//TODO: implement callback
	}

}


impl<'a> i2c::I2CHwSlaveClient for PortSignpostTock <'a> {
	fn command_complete(&self, 
						buffer: &'static mut [u8], 
						length: u8,
						transmission_type: hil::i2c::SlaveTransmissionType) {
		//TODO: implement callback
	}

	fn read_expected(&self) {
		//TODO:	
	}
	
	fn write_expected(&self) {
		//TODO:
	}

}



/*
impl<'a, A: time::Alarm + 'a> i2c::I2CClient for PortSignpostTock<'a, A> {
	// Link from I2C capsule to PortSignpostTock capsule
	// fn command_complete ()
}

impl<'a, A: time::Alarm + 'a> time::Client for PortSignpostTock<'a, A> [
	// Link from time capsule to PortSignpostTock capsule
	// fn fired ()
}

*/


