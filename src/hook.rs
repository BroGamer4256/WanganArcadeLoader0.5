// Retour didnt work so im doing it all myself
// This solution sucks btw

use libc::*;
use std::ffi::CString;

pub unsafe fn hook_symbol(symbol: &str, func: *const ()) {
	hook(get_symbol(symbol), func);
}

pub unsafe fn get_symbol(symbol: &str) -> *mut () {
	let symbol = CString::new(symbol).unwrap();
	let module = dlopen(std::ptr::null(), RTLD_LAZY);
	let address = dlsym(module, symbol.as_ptr());
	dlclose(module);
	address as *mut ()
}

pub unsafe fn hook(address: *mut (), func: *const ()) {
	let func = func as usize;
	let func = func.to_le_bytes();
	/*
		mov eax, func
		jmp eax
	*/
	let mut data = Vec::with_capacity(7);
	data.push(0xB8);
	for i in func {
		data.push(i);
	}
	data.push(0xFF);
	data.push(0xE0);
	write_memory(address, &data);
}

pub unsafe fn write_memory(address: *mut (), data: &[u8]) {
	region::protect(address, data.len(), region::Protection::READ_WRITE_EXECUTE).unwrap();
	std::ptr::copy_nonoverlapping(data.as_ptr(), address as *mut u8, data.len());
}
