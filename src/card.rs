use std::io::Write;
use std::sync::Mutex;

use crate::*;
const INIT: u8 = 0x10;
const READSTATUS: u8 = 0x20;
const CANCEL: u8 = 0x40;
const EJECT: u8 = 0x80;
const DISPENSECARD: u8 = 0xB0;
const PRINTSETTING: u8 = 0x78;
const READ: u8 = 0x33;
const WRITE: u8 = 0x53;

pub static mut CARD_DATA: Mutex<Vec<u8>> = Mutex::new(Vec::new());

// +0x06: Status
unsafe extern "C" fn exec(card_printer: *mut u32) {
	let has_command = card_printer.read();
	if has_command == 0 {
		return;
	}
	let request = card_printer.byte_add(0x08).read() as *mut u8;
	let start = request.byte_add(0).read();
	if start != 2 {
		dbg!(start);
		return;
	}
	let count = request.byte_add(1).read();
	let command = request.byte_add(2).read();
	let mut data = Vec::new();
	for i in 6..count {
		data.push(request.byte_add(i as usize).read())
	}

	match command {
		INIT => card_printer.write(0x00),
		READSTATUS => card_printer.write(0x00),
		DISPENSECARD => {
			let check = data[0] == 0x32;
			card_printer.write(0x00);
			if check {
				card_printer.byte_add(0x06).write(0x37);
			} else {
				card_printer.byte_add(0x04).write(0x33);
			}
		}
		READ => {
			// Just assume it always reads all 3 tracks
			card_printer.write(0x00);
			card_printer.byte_add(0x04).write(0x31);
			if data[0] == 0x32 {
				if CARD_DATA.lock().unwrap().len() == 0 {
					card_printer.byte_add(0x04).write(0x30);
					card_printer.byte_add(0x06).write(0x34);
				}
				return;
			}
			let write_buf = card_printer.byte_add(0x10).read() as *mut u8;
			write_buf.write(0x00);
			write_buf.byte_add(0x04).write(0x33);
			write_buf.byte_add(0x05).write(0x30);
			write_buf.byte_add(0x06).write(0x30);
			for (i, data) in CARD_DATA.get_mut().unwrap().iter().enumerate() {
				write_buf.byte_add(i + 0x06).write(*data);
			}
		}
		WRITE => {
			let card_data = CARD_DATA.get_mut().unwrap();
			card_printer.write(0x00);
			if data[2] == 0x30 {
				// track 1
				for (i, data) in data.iter().skip(3).enumerate() {
					card_data[i] = *data;
				}
			} else if data[2] == 0x35 {
				// track 23
				for (i, data) in data.iter().skip(3).enumerate() {
					card_data[i + 0x45] = *data;
				}
			} else if data[2] == 0x36 {
				// track 123
				card_data.clear();
				for data in data.iter().skip(3) {
					card_data.push(*data);
				}
			} else {
				panic!("Unknown track combination {}", data[2]);
			}
			let mut file = std::fs::File::create("card.bin").unwrap();
			file.write(&card_data).unwrap();
		}
		CANCEL => card_printer.write(0x00),
		EJECT => {
			CARD_DATA.get_mut().unwrap().clear();
			card_printer.write(0x00);
			card_printer.byte_add(0x04).write(0x30);
		}
		PRINTSETTING => card_printer.write(0x00),

		_ => panic!("Unhandled command {:#0x}", command as u8),
	}
}

pub unsafe fn init() {
	hook::hook_symbol("_ZN13clCardPrinter4execEv", exec as *const ());
}
