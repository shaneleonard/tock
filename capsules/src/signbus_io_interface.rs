/// Kernel implementation of signbus_io_interface
/// apps/libsignpost/signbus_io_interface.c -> kernel/tock/capsules/src/signbus_io_interface.rs
/// By: Justin Hsieh

use core::cell::Cell;
use core::cmp;
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil;
use kernel::hil::gpio;
use kernel::hil::time;
use port_signpost_tock;

pub static mut BUFFER0: [u8; 256] = [0; 256];
pub static mut BUFFER1: [u8; 256] = [0; 256];


struct SignbusNetworkFlags {
	is_fragment:	Cell<bool>,
	is_encrypted:	Cell<bool>,
	rsv_wire_bit5:	Cell<bool>,
	rsv_wire_bit4:	Cell<bool>,
	version:		Cell<u8>,
}

struct SignbusNetworkHeader {
	flags:				SignbusNetworkFlags,
	src:				Cell<u8>,
	sequence_number:	Cell<u16>,
	length:				Cell<u16>,
	fragment_offset:	Cell<u16>,
}

pub struct Packet {
	header: SignbusNetworkHeader,
	data:	TakeCell<'static, [u8]>,	
}

pub struct SignbusIOInterface<'a> {
	port_signpost_tock: 	&'a port_signpost_tock::PortSignpostTock<'a>,
	this_device_address:	Cell<u8>,
	sequence_number:		Cell<u16>,
	slave_write_buf:		TakeCell <'static, [u8]>,
	packet_buf:				TakeCell <'static, [u8]>,
}

impl<'a> SignbusIOInterface<'a,> {
	pub fn new(port_signpost_tock: &'a port_signpost_tock::PortSignpostTock<'a>,
				slave_write_buf:	&'static mut [u8],
				packet_buf: 		&'static mut [u8]) -> SignbusIOInterface <'a> {
	
		SignbusIOInterface {
			port_signpost_tock:  	port_signpost_tock,
			this_device_address:	Cell::new(0),
			sequence_number:		Cell::new(0),
			slave_write_buf:		TakeCell::new(slave_write_buf),
			packet_buf:				TakeCell::new(packet_buf),
		}
	}

	fn htons(&self, a: u16) -> u16 {
		return (((a & 0x00FF) << 8) | ((a & 0xFF00) >> 8));
	}

	pub fn signbus_io_init(&self, address: u8) -> ReturnCode {
		self.this_device_address.set(address);
		self.port_signpost_tock.init(address);
			
		debug!("Address: {}", self.this_device_address.get());

		ReturnCode::SUCCESS
	}

	pub fn signbus_io_send(&self, dest: u8, encrypted: bool, 
							data: &'static mut [u8],
							len: u32) -> ReturnCode {
		
		self.sequence_number.set(self.sequence_number.get() + 1);
		let toSend = len;

		// TODO: MACRO
		//let MAX_DATA_LEN = I2C_MAX_LEN-sizeof(SignbusNetworkHeader);
		
		let MAX_DATA_LEN = 1;

		if (len % MAX_DATA_LEN == 0) {
			let numPackets = (len/MAX_DATA_LEN) + 1;
		} 
		else {
			let numPackets = (len/MAX_DATA_LEN);
		}

		let flags: SignbusNetworkFlags = SignbusNetworkFlags {
			is_fragment:	Cell::new(false), // toSend > MAX_DATA_LEN
			is_encrypted:	Cell::new(encrypted),
			rsv_wire_bit5:	Cell::new(false),
			rsv_wire_bit4:	Cell::new(false),
			version:		Cell::new(0x1),
		};

		let header: SignbusNetworkHeader = SignbusNetworkHeader {
			flags:				flags,
			src:				Cell::new(self.this_device_address.get()),
			sequence_number:	Cell::new(self.htons(self.sequence_number.get())),
			length:				Cell::new(len as u16),
			fragment_offset:	Cell::new((len-toSend) as u16),
		};

		let packet: Packet = Packet {
			header: header,
			data:	TakeCell::new(data),	
		};


		ReturnCode::SUCCESS
	}

	
}
