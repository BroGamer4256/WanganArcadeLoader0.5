use libc::*;
use std::ffi::CString;

#[no_mangle]
pub unsafe extern "C" fn hook_symbol(symbol: &str, func: *const ()) -> *const () {
	hook(get_symbol(symbol), func)
}

#[no_mangle]
pub unsafe extern "C" fn get_symbol(symbol: &str) -> *mut () {
	let symbol = CString::new(symbol).unwrap();
	let module = dlopen(std::ptr::null(), RTLD_LAZY);
	let address = dlsym(module, symbol.as_ptr());
	dlclose(module);
	address as *mut ()
}

#[no_mangle]
pub unsafe extern "C" fn hook(address: *mut (), func: *const ()) -> *const () {
	let hook = retour::RawDetour::new(address, func).unwrap();
	hook.enable().unwrap();
	let trampoline = hook.trampoline() as *const ();
	std::mem::forget(hook);
	trampoline
}

#[no_mangle]
pub unsafe extern "C" fn write_memory(address: *mut (), data: &[u8]) {
	region::protect(address, data.len(), region::Protection::READ_WRITE_EXECUTE).unwrap();
	std::ptr::copy_nonoverlapping(data.as_ptr(), address as *mut u8, data.len());
}
