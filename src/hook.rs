use libc::*;
use std::ffi::CString;

pub unsafe fn hook_symbol(symbol: &str, func: *const ()) -> *const () {
	hook(get_symbol(symbol), func)
}

pub unsafe fn get_symbol(symbol: &str) -> *mut () {
	let symbol = CString::new(symbol).unwrap();
	let module = dlopen(std::ptr::null(), RTLD_LAZY);
	let address = dlsym(module, symbol.as_ptr());
	dlclose(module);
	address as *mut ()
}

pub unsafe fn hook(address: *mut (), func: *const ()) -> *const () {
	let Ok(hook) = retour::RawDetour::new(address, func) else {
		return std::ptr::null();
	};
	let Ok(_) = hook.enable() else {
		return std::ptr::null();
	};
	let trampoline = hook.trampoline() as *const ();
	std::mem::forget(hook);
	trampoline
}

pub unsafe fn write_memory(address: *mut (), data: &[u8]) {
	region::protect(address, data.len(), region::Protection::READ_WRITE_EXECUTE).unwrap();
	std::ptr::copy_nonoverlapping(data.as_ptr(), address as *mut u8, data.len());
}
