/// Kernel implementation of signbus_io_interface
/// apps/libsignpost/signbus_io_interface.c -> kernel/tock/capsules/src/signbus_io_interface.rs
/// By: Justin Hsieh


use core::mem;
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

//static EXAMPLE: usize = mem::size_of::<SignbusNetworkFlags>();
const I2C_MAX_LEN: u16 = 255; 
const SIZE_FLAGS: u16 = 1;
const SIZE_HEADER: u16 = 8;



#[repr(C, packed)]
pub struct SignbusNetworkFlags {
	is_fragment:	bool,
	is_encrypted:	bool,
	rsv_wire_bit5:	bool,
	rsv_wire_bit4:	bool,
	version:		u8,
}

#[repr(C, packed)]
pub struct SignbusNetworkHeader {
	flags:				SignbusNetworkFlags,
	src:				u8,
	sequence_number:	u16,
	length:				u16,
	fragment_offset:	u16,
}


#[repr(C, packed)]
pub struct Packet {
	header: SignbusNetworkHeader,
	data:	&'static mut [u8],	
}


fn serialize<T: Sized>(p: &T) -> &[u8] {
    	::core::slice::from_raw_parts((p as *const T) as *const u8, 
										::core::mem::size_of::<T>())
}


pub struct SignbusIOInterface<'a> {
	port_signpost_tock: 	&'a port_signpost_tock::PortSignpostTock<'a>,
	this_device_address:	Cell<u8>,
	sequence_number:		Cell<u16>,
	slave_write_buf:		TakeCell <'static, [u8]>,
	packet_buf:				TakeCell <'static, [u8]>,
}

impl<'a> SignbusIOInterface<'a> {
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


	// Host-to-network short (packages certain data into header)
	fn htons(&self, a: u16) -> u16 {
		return ((a as u16 & 0x00FF) << 8) | ((a as u16 & 0xFF00) >> 8);
	}

	/// Initialization routine to set up the slave address for this device.
	/// MUST be called before any other methods.
	pub fn signbus_io_init(&self, address: u8) -> ReturnCode {
		
		self.this_device_address.set(address);
		self.port_signpost_tock.init(address);
			
		debug!("Address: {}", self.this_device_address.get());
		debug!("SignbusNetworkHeader: {}", mem::size_of::<SignbusNetworkHeader>());
		debug!("SignbusNetworkFlags: {}", mem::size_of::<SignbusNetworkFlags>());

		return ReturnCode::SUCCESS;
	}


	// synchronous send call
	pub fn signbus_io_send(&self, 
							dest: u8, 
							encrypted: bool, 
							data: &'static mut [u8],
							len: u16) -> ReturnCode {

		let mut toSend = len;
		
		// calculate max data length
		let MAX_DATA_LEN: u16 = I2C_MAX_LEN - (mem::size_of::<SignbusNetworkHeader>()as u16);
	

		// calculate the number of packets we will have to send
		let mut numPackets: u16 = len/MAX_DATA_LEN + 1;		
		if len % MAX_DATA_LEN == 0 {
			numPackets = numPackets - 1; 
		}
		
		// update sequence number
		self.sequence_number.set(self.sequence_number.get() + 1);

		// Network flags
		let flags: SignbusNetworkFlags = SignbusNetworkFlags {
			is_fragment:	false, // toSend > MAX_DATA_LEN
			is_encrypted:	encrypted,
			rsv_wire_bit5:	false,
			rsv_wire_bit4:	false,
			version:		0x1,
		};

		// Network header
		let header: SignbusNetworkHeader = SignbusNetworkHeader {
			flags:				flags,
			src:				self.this_device_address.get(),
			sequence_number:	self.htons(self.sequence_number.get() as u16),
			length:				self.htons(numPackets * mem::size_of::<SignbusNetworkHeader>() as u16 + len),
			fragment_offset:	(len-toSend) as u16,
		};

		// Packet
		let mut packet: Packet = Packet {
			header: header,
			data:	data,	
		};


		while toSend > 0 {
			let morePackets: bool = toSend > MAX_DATA_LEN;;
			let offset: u16 = len - toSend;

			packet.header.flags.is_fragment = morePackets;
		
			packet.header.fragment_offset = self.htons(offset);

			if morePackets == true {
				self.port_signpost_tock.master_tx_buffer.take().map(|port_buffer|{
					let d = &mut port_buffer.as_mut()[0..MAX_DATA_LEN as usize];
						

					});
			}
			else {
				self.port_signpost_tock.master_tx_buffer.take().map(|port_buffer|{
					let d = &mut port_buffer.as_mut()[0..toSend as usize];
				});
			}

			if morePackets == true {
				let rc = self.port_signpost_tock.i2c_master_write(dest, I2C_MAX_LEN);	
				
				if rc != ReturnCode::SUCCESS { return rc; } 

				toSend -= MAX_DATA_LEN;
			}
			else {
				let rc = self.port_signpost_tock.i2c_master_write(dest, toSend);	
				
				if rc != ReturnCode::SUCCESS { return rc; } 

				toSend = 0;

			}
	

		}


		ReturnCode::SUCCESS
	}

	
}
