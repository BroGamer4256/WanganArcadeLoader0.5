use crate::*;


#[repr(C)]
pub struct PortSettings {
	unk_0: c_int, // unknown param
	errno: c_int, // error number
	fd: c_int, // file_descriptor
	unk_c: c_int, // another unknown param
	portno: c_int // port number e.g /dev/ttyMX
/*
	unk_14: c_int,
	unk_18: c_int,
	unk_1c: c_int,
	unk_20: c_int
*/
}

unsafe extern "C" fn portopen(ps: *mut PortSettings) -> c_int
{	
	let path: String = CONFIG.card_device.clone(); // get path from config.toml
	let mut reader_path = CString::new(path).unwrap(); // create a new CString using above String.
	if reader_path.is_empty(){ // if no path from config toml
		println!("Reader path not set, please check config.toml"); // throw error
		reader_path = CString::new("/dev/ttyM0").unwrap(); // use ttyM0 as default
	}
	ps.as_mut().unwrap().fd = open(reader_path.as_ptr(), 0x101902); // 0x902 from binary, feed open file descriptor into above struct
	if ps.as_ref().unwrap().fd < 0{ // if the file descriptor less than 0, e.g. -1
		return 0; // return 0 to function
	}
	else{ // if fd more than or eq 0
		return 1; // return 1
	}
}

pub unsafe fn init() { // default init function
	hook::hook_symbol("_ZN10clSerialN24openEv", portopen as *const ()); // hooks onto symbol in binary.
}