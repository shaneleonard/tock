/// Kernel implementation of signbus_io_interface
/// apps/libsignpost/signbus_io_interface.c -> kernel/tock/capsules/src/signbus_io_interface.rs
/// By: Justin Hsieh

use core::cell::Cell;
use core::cmp;
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil;
use kernel::hil::gpio;
use kernel::hil::i2c;
use kernel::hil::time;


pub struct SignbusIOInterface<'a, A: time::Alarm + 'a> {
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

impl<'a, A: time::Alarm + 'a> SignbusIOInterface<'a, A> {
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
}
