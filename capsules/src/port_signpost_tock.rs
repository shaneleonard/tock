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

pub struct App {
    callback: 			Option<Callback>,
    master_tx_buffer: 	Option<AppSlice<Shared, u8>>,
    master_rx_buffer: 	Option<AppSlice<Shared, u8>>,
    slave_tx_buffer: 	Option<AppSlice<Shared, u8>>,
    slave_rx_buffer: 	Option<AppSlice<Shared, u8>>,
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

#[derive(Clone,Copy,PartialEq)]
enum MasterAction {
    Read(u8),
    Write,
}

pub struct I2CMasterSlaveDriver<'a> {
    i2c: &'a hil::i2c::I2CMasterSlave,
    listening: Cell<bool>,
    master_action: Cell<MasterAction>, // Whether we issued a write or read as master
    master_buffer: TakeCell<'static, [u8]>,
    slave_buffer1: TakeCell<'static, [u8]>,
    slave_buffer2: TakeCell<'static, [u8]>,
    app: MapCell<App>,
}

impl<'a> I2CMasterSlaveDriver<'a> {
    pub fn new(i2c: &'a hil::i2c::I2CMasterSlave,
               master_buffer: &'static mut [u8],
               slave_buffer1: &'static mut [u8],
               slave_buffer2: &'static mut [u8])
               -> I2CMasterSlaveDriver<'a> {
        I2CMasterSlaveDriver {
            i2c: i2c,
            listening: Cell::new(false),
            master_action: Cell::new(MasterAction::Write),
            master_buffer: TakeCell::new(master_buffer),
            slave_buffer1: TakeCell::new(slave_buffer1),
            slave_buffer2: TakeCell::new(slave_buffer2),
            app: MapCell::new(App::default()),
        }
    }
}


impl<'a> hil::i2c::I2CHwMasterClient for I2CMasterSlaveDriver<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], error: hil::i2c::Error) {

        // Map I2C error to a number we can pass back to the application
        let err: isize = match error {
            hil::i2c::Error::AddressNak => -1,
            hil::i2c::Error::DataNak => -2,
            hil::i2c::Error::ArbitrationLost => -3,
            hil::i2c::Error::CommandComplete => 0,
        };

        // Signal the application layer. Need to copy read in bytes if this
        // was a read call.
        match self.master_action.get() {
            MasterAction::Write => {
                self.master_buffer.replace(buffer);

                self.app.map(|app| {
                    app.callback.map(|mut cb| { cb.schedule(0, err as usize, 0); });
                });
            }

            MasterAction::Read(read_len) => {
                self.app.map(|app| {
                    app.master_rx_buffer.as_mut().map(move |app_buffer| {
                        let len = cmp::min(app_buffer.len(), read_len as usize);

                        let d = &mut app_buffer.as_mut()[0..(len as usize)];
                        for (i, c) in buffer[0..len].iter().enumerate() {
                            d[i] = *c;
                        }

                        self.master_buffer.replace(buffer);
                    });

                    app.callback.map(|mut cb| { cb.schedule(1, err as usize, 0); });
                });
            }
        }

        // Check to see if we were listening as an I2C slave and should re-enable
        // that mode.
        if self.listening.get() {
            hil::i2c::I2CSlave::enable(self.i2c);
            hil::i2c::I2CSlave::listen(self.i2c);
        }
    }
}

impl<'a> hil::i2c::I2CHwSlaveClient for I2CMasterSlaveDriver<'a> {
    fn command_complete(&self,
                        buffer: &'static mut [u8],
                        length: u8,
                        transmission_type: hil::i2c::SlaveTransmissionType) {

        // Need to know if read or write
        //   - on write, copy bytes to app slice and do callback
        //     then pass buffer back to hw driver
        //   - on read, just signal upper layer and replace the read buffer
        //     in this driver

        match transmission_type {
            hil::i2c::SlaveTransmissionType::Write => {
                self.app.map(|app| {
                    app.slave_rx_buffer.as_mut().map(move |app_rx| {
                        // Check bounds for write length
                        let buf_len = cmp::min(app_rx.len(), buffer.len());
                        let read_len = cmp::min(buf_len, length as usize);

                        let d = &mut app_rx.as_mut()[0..read_len];
                        for (i, c) in buffer[0..read_len].iter_mut().enumerate() {
                            d[i] = *c;
                        }

                        self.slave_buffer1.replace(buffer);
                    });

                    app.callback.map(|mut cb| { cb.schedule(3, length as usize, 0); });
                });
            }

            hil::i2c::SlaveTransmissionType::Read => {
                self.slave_buffer2.replace(buffer);

                // Notify the app that the read finished
                self.app.map(|app| {
                    app.callback.map(|mut cb| { cb.schedule(4, length as usize, 0); });
                });
            }
        }
    }

    fn read_expected(&self) {
        // Pass this up to the client. Not much we can do until the application
        // has setup a buffer to read from.
        self.app.map(|app| {
            app.callback.map(|mut cb| {
                // Ask the app to setup a read buffer. The app must call
                // command 3 after it has setup the shared read buffer with
                // the correct bytes.
                cb.schedule(2, 0, 0);
            });
        });
    }

    fn write_expected(&self) {
        // Don't expect this to occur. We will typically have a buffer waiting
        // to receive bytes because this module has a buffer and may as well
        // just let the hardware layer have it. But, if it does happen
        // we can respond.
        self.slave_buffer1
            .take()
            .map(|buffer| { hil::i2c::I2CSlave::write_receive(self.i2c, buffer, 255); });
    }
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

}


pub struct PortSignpostTock<'a, A: time::Alarm + 'a> {
	i2c: 		&'a i2c::I2CDevice,
	alarm:		&'a A,
	callback:	Cell<Option<Callback>>,
	state:		Cell<State>,
	buffer:		TakeCell <'static, [u8]>,
}

impl<'a, A: time::Alarm + 'a> PortSignpostTock<'a, A> {
	pub fn new(i2c: &'a I2CMasterSlaveDriver, alarm: &'a A, buffer: &'static mut [u8]) -> PortSignpostTock<'a, A> {

		PortSignpostTock {
			i2c:  		i2c,
			alarm: 		alarm,
			callback: 	Cell::new(None),
			state:		Cell::new(State::Idle),
			buffer:		TakeCell::new(buffer),	
		}
	}

	pub fn init(&self, uint8_t i2c_address) {
			
	}

	pub fn set_master_write_buffer(&self) {
		i2c.
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




