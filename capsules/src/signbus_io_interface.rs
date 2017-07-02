/// Kernel implementation of signbus_io_interface
/// apps/libsignpost/signbus_io_interface.c -> kernel/tock/capsules/src/signbus_io_interface.rs
/// By: Justin Hsieh


use core::mem;
use core::slice;
use core::cell::Cell;
//use core::cmp;
use kernel::{ReturnCode};
use kernel::common::take_cell::{MapCell, TakeCell};
//use kernel::hil;
//use kernel::hil::gpio;
//use kernel::hil::time;
use port_signpost_tock;

pub static mut BUFFER0: [u8; 256] = [0; 256];
pub static mut BUFFER1: [u8; 256] = [0; 256];
pub static mut BUFFER2: [u8; 256] = [1; 256];

static debug: u8 = 1;
const I2C_MAX_LEN: u16 = 255; 
const SIZE_FLAGS: u16 = 1;
const SIZE_HEADER: u16 = 8;



#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct SignbusNetworkFlags {
	is_fragment:	bool,
	is_encrypted:	bool,
	rsv_wire_bit5:	bool,
	rsv_wire_bit4:	bool,
	version:		u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct SignbusNetworkHeader {
	flags:				SignbusNetworkFlags,
	src:				u8,
	sequence_number:	u16,
	length:				u16,
	fragment_offset:	u16,
}


#[repr(C, packed)]
//#[derive(Clone, Copy, Debug)]
pub struct Packet {
	header: SignbusNetworkHeader,
	data:	&'static mut [u8],	
}

unsafe fn as_byte_slice<'a, T>(input: &'a T) -> &'a [u8] {
    slice::from_raw_parts(input as *const T as *const u8, mem::size_of::<T>())
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
		
		if debug == 1 {	
			//debug!("Address: {}", self.this_device_address.get());
			//debug!("SignbusNetworkHeader: {}", mem::size_of::<SignbusNetworkHeader>());
			//debug!("SignbusNetworkFlags: {}", mem::size_of::<SignbusNetworkFlags>());
		}

		return ReturnCode::SUCCESS;
	}


	// synchronous send call
	pub fn signbus_io_send(&self, 
							dest: u8, 
							encrypted: bool, 
							data: &'static mut [u8],
							len: u16) -> ReturnCode {

		// update sequence number
		self.sequence_number.set(self.sequence_number.get() + 1);
		
		// number of bytes left to be sent
		let mut toSend: u16 = len;
		// size of header (sent everytime)
		let mut HEADER_SIZE: u16 = mem::size_of::<SignbusNetworkHeader>() as u16;
		// max data in one packet
		let MAX_DATA_LEN: u16 = I2C_MAX_LEN - HEADER_SIZE;
		// number of packets to be sent
		let mut numPackets: u16 = len/MAX_DATA_LEN + 1;		
		if len % MAX_DATA_LEN == 0 {
			numPackets = numPackets - 1; 
		}
	
		// Network Flags
		let flags: SignbusNetworkFlags = SignbusNetworkFlags {
			is_fragment:	false, // toSend > MAX_DATA_LEN
			is_encrypted:	encrypted,
			rsv_wire_bit5:	false,
			rsv_wire_bit4:	false,
			version:		0x1,
		};

		// Network Header
		let header: SignbusNetworkHeader = SignbusNetworkHeader {
			flags:				flags,
			src:				self.this_device_address.get(),
			sequence_number:	self.htons(self.sequence_number.get() as u16),
			length:				self.htons(numPackets * (HEADER_SIZE + len)),
			fragment_offset:	0 as u16,
		};

		// Packet
		let mut packet: Packet = Packet {
			header: header,
			data:	data,	
		};
		
		if debug == 1 {
			//debug!("{:?}", data.len());
			//debug!("{:?}", packet.data.len());
			//debug!("data length: {} ", packet.data.len());
			//debug!("toSend: {} ", toSend);
			//debug!("MAX_DATA_LEN: {} ", MAX_DATA_LEN);
			//debug!("HEADER_SIZE: {} ", HEADER_SIZE);
			//debug!("numPackets: {} ", numPackets);
		}
		
		
		while toSend > 0 {
			let morePackets: bool = toSend > MAX_DATA_LEN;;
			let mut START: usize = (HEADER_SIZE + 1) as usize; 

			// UPDATE HEADER
			packet.header.flags.is_fragment = morePackets;
			packet.header.fragment_offset = self.htons(len-toSend);

			// COPY HEADER
			self.port_signpost_tock.master_tx_buffer.map(|port_buffer|{
				let d = &mut port_buffer.as_mut()[0..HEADER_SIZE as usize];
				let bytes: &[u8]= unsafe { as_byte_slice(&packet.header) };
				for (i, c) in bytes[0..HEADER_SIZE as usize].iter().enumerate() {
					d[i] = *c;
				}
			});

			// COPY DATA
			if morePackets == true {
				self.port_signpost_tock.master_tx_buffer.map(|port_buffer|{
					let d = &mut port_buffer.as_mut()[START..I2C_MAX_LEN as usize];
					for (i, c) in packet.data[0..(MAX_DATA_LEN-1) as usize].iter().enumerate() {
						d[i] = *c;
					}	
				});
			}
			else {
				self.port_signpost_tock.master_tx_buffer.map(|port_buffer|{
					let d = &mut port_buffer.as_mut()[START..(START as u16 +toSend) as usize];
					for (i, c) in packet.data[0..toSend as usize].iter().enumerate() {
						d[i] = *c;
					}	
				});
			}
		
			// SEND I2C message and update bytes left to send
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

		if debug == 1 {
			//debug!("End of Function");
		}

		ReturnCode::SUCCESS
	}
}
