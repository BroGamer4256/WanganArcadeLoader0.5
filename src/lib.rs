use libc::*;
use std::ffi::{CStr, CString};

pub mod adm;
pub mod gl;
pub mod hook;
pub mod jamma;

extern "C" fn adachi() -> c_int {
	true as c_int
}

#[no_mangle]
unsafe extern "C" fn system(command: *const c_char) -> c_int {
	let cstr = CStr::from_ptr(command);
	let str = cstr.to_str().unwrap();
	dbg!(str);
	if str.starts_with("perl") {
		if str.ends_with("/tmp/ifconfig.txt") {
			0
		} else {
			0
		}
	} else {
		let system = CString::new("system").unwrap();
		let _original = dlsym(RTLD_NEXT, system.as_ptr());
		0
	}
}

// Redirect clLog to std::cout
static CL_MAIN_ORIGINAL: [u8; 19] = [
	0xB8, 0xE4, 0x5D, 0x8C, 0x08, 0x55, 0x89, 0xE5, 0x57, 0x56, 0x53, 0x81, 0xEC, 0x8C, 0x00, 0x00,
	0x00, 0xFF, 0xE0,
];

static mut CL_MAIN_IMPLEMENTATION: [u8; 28] = [
	0x8B, 0x5C, 0x24, 0x04, 0x83, 0xEC, 0x08, 0x53, 0xB8, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xD0, 0x83,
	0xC4, 0x0C, 0x53, 0xB8, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xD0, 0x5B, 0xC3,
];

unsafe extern "C" fn cl_main(address: *mut ()) {
	hook::write_memory(
		address,
		&(hook::get_symbol("_ZSt4cout") as usize).to_le_bytes(),
	);
}

unsafe fn hook_cl_main() {
	let original = CL_MAIN_ORIGINAL.as_ptr();
	region::protect(
		original,
		CL_MAIN_ORIGINAL.len(),
		region::Protection::READ_WRITE_EXECUTE,
	)
	.unwrap();

	for (i, data) in (original as usize).to_le_bytes().iter().enumerate() {
		CL_MAIN_IMPLEMENTATION[i + 9] = *data;
	}
	let func = cl_main as *const () as usize;
	for (i, data) in func.to_le_bytes().iter().enumerate() {
		CL_MAIN_IMPLEMENTATION[i + 20] = *data;
	}

	let implementation = CL_MAIN_IMPLEMENTATION.as_ptr();
	region::protect(
		implementation,
		CL_MAIN_IMPLEMENTATION.len(),
		region::Protection::READ_WRITE_EXECUTE,
	)
	.unwrap();

	hook::hook(
		hook::get_symbol("_ZN6clMainC1Ev"),
		implementation as *const (),
	);
}

#[ctor::ctor]
unsafe fn init() {
	let exe = std::env::current_exe().unwrap();
	if !exe.ends_with("main") {
		panic!("Not 3DX+");
	}
	hook::hook_symbol("_ZNK6clHaspcvbEv", adachi as *const ());
	hook::hook_symbol("_ZNK7clHasp2cvbEv", adachi as *const ());
	hook::hook_symbol("_ZN18clSeqBootNetThread3runEPv", adachi as *const ());
	adm::init();
	jamma::init();
	hook_cl_main();
}
