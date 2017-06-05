/// Signbus implementation in the kernel level

use core::cell::Cell;
use core::cmp;
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil;


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

