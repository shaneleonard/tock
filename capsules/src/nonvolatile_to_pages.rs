//! Map arbitrary nonvolatile reads and writes to page operations.
//!
//! ```
//! hil::nonvolatile_storage::NonvolatileStorage
//! ┌──────────────────────────────────────────┐
//! │                                          │
//! │                This module               │
//! │                                          │
//! └──────────────────────────────────────────┘
//!               hil::flash::Flash
//! ```

use core::cmp;
use core::cell::Cell;
use kernel::{AppId, AppSlice, Callback, Container, Driver, ReturnCode, Shared};
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use kernel::process::Error;

#[derive(Clone,Copy,Debug,PartialEq)]
enum State {
    Idle,

    // Doing a read operation.
    Read,

}

pub struct NonvolatileToPages<'a, F: hil::flash::Flash + 'static> {
    driver: &'a F,
    client: Cell<Option<&'static hil::nonvolatile_storage::NonvolatileStorageClient>>,
    pagebuffer: TakeCell<'static, F::Page>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    address: Cell<usize>,
    length: Cell<usize>,
    remaining_length: Cell<usize>,
    buffer_index: Cell<usize>,
}

impl<'a, F: hil::flash::Flash + 'a> NonvolatileToPages<'a, F> {
    pub fn new(driver: &'a F,
               buffer: &'static mut F::Page)
               -> NonvolatileToPages<'a, F> {
        NonvolatileToPages {
            driver: driver,
            client: Cell::new(None),
            pagebuffer: TakeCell::new(buffer),
            state: Cell::new(State::Idle),
            buffer: TakeCell::empty(),
            address: Cell::new(0),
            length: Cell::new(0),
            remaining_length: Cell::new(0),
            buffer_index: Cell::new(0),
        }
    }
}

impl<'a, F: hil::flash::Flash + 'a> hil::nonvolatile_storage::NonvolatileStorage for NonvolatileToPages<'a, F> {
    fn set_client(&self, client: &'static hil::nonvolatile_storage::NonvolatileStorageClient) {
        self.client.set(Some(client));
    }

    fn read(&self, buffer: &'static mut [u8], address: usize, length: usize) -> ReturnCode {
        if self.state.get() != State::Idle {
            return ReturnCode::EBUSY;
        }

        self.pagebuffer.take().map_or(ReturnCode::ERESERVE, move |pagebuffer| {
            let page_size = pagebuffer.as_mut().len();

            // Just start reading. We'll worry about how much of the page we
            // want later.
            self.state.set(State::Read);
            self.buffer.replace(buffer);
            self.address.set(address);
            self.length.set(length);
            self.remaining_length.set(length);
            self.buffer_index.set(0);
            self.driver.read_page(address / page_size, pagebuffer)
        })
    }

    fn write(&self, buffer: &'static mut [u8], address: usize, length: usize) -> ReturnCode {
        if self.state.get() != State::Idle {
            return ReturnCode::EBUSY;
        }

        ReturnCode::SUCCESS
    }
}



impl<'a, F: hil::flash::Flash + 'a> hil::flash::Client<F> for NonvolatileToPages<'a, F> {
    fn read_complete(&self, pagebuffer: &'static mut F::Page, _error: hil::flash::Error) {

        match self.state.get() {
            State::Read => {
                // OK we got a page from flash. Copy what we actually want from it
                // out of it.
                self.buffer.take().map(move |buffer| {
                    let page_size = pagebuffer.as_mut().len();
                    // This will get us our offset into the page.
                    let page_index = self.address.get() % page_size;
                    // Length is either the rest of the page or how much we have left.
                    let len = cmp::min(page_size - page_index, self.remaining_length.get());
                    // And where we left off in the user buffer.
                    let buffer_index = self.buffer_index.get();

                    // Copy what we read from the page buffer to the user buffer.
                    for i in 0..len {
                        buffer[buffer_index + i] = pagebuffer.as_mut()[page_index + i];
                    }

                    // Decide if we are done.
                    let new_len = self.remaining_length.get() - len;
                    if new_len == 0 {
                        // Nothing more to do. Put things back and issue callback.
                        self.pagebuffer.replace(pagebuffer);
                        self.state.set(State::Idle);
                        self.client.get().map(move |client| client.read_done(buffer, self.length.get()));
                    } else {
                        // More to do!
                        self.buffer.replace(buffer);
                        // Increment all buffer pointers and state.
                        self.remaining_length.set(self.remaining_length.get() - len);
                        self.address.set(self.address.get() + len);
                        self.buffer_index.set(buffer_index + len);
                        self.driver.read_page(self.address.get() / page_size, pagebuffer);
                    }
                });
            }
            _ => {}
        }

    }

    fn write_complete(&self, buffer: &'static mut F::Page, error: hil::flash::Error) {

    }

    fn erase_complete(&self, error: hil::flash::Error) {}
}
