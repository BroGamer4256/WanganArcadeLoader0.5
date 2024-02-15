use crate::*;
use std::ffi::CString;

const FUNCS: [&'static str; 69] = [
	"alcSuspendContext",
	"alcCloseDevice",
	"alListener3f",
	"alGetFloat",
	"alcGetString",
	"alIsExtensionPresent",
	"alcIsExtensionPresent",
	"alGetBooleanv",
	"alDopplerFactor",
	"alSourcePausev",
	"alDisable",
	"alcGetCurrentContext",
	"alEnable",
	"alDeleteBuffers",
	"alGetSourcef",
	"alSourcefv",
	"alGetIntegerv",
	"alGetDouble",
	"alGetEnumValue",
	"alSourcei",
	"alSourceRewind",
	"alcDestroyContext",
	"alGetSourcefv",
	"alGetBufferi",
	"alSourcePlay",
	"alSourcef",
	"alSourceStop",
	"alcGetError",
	"alGetSource3f",
	"alSource3f",
	"alGetListenerfv",
	"alGetError",
	"alIsBuffer",
	"alcGetContextsDevice",
	"alGetListener3f",
	"alcGetIntegerv",
	"alDopplerVelocity",
	"alSourcePlayv",
	"alSourceUnqueueBuffers",
	"alGetFloatv",
	"alcOpenDevice",
	"alcProcessContext",
	"alListeneri",
	"alListenerfv",
	"alDistanceModel",
	"alSourcePause",
	"alGenSources",
	"alIsEnabled",
	"alcMakeContextCurrent",
	"alDeleteSources",
	"alcGetEnumValue",
	"alSourceStopv",
	"alGetProcAddress",
	"alSourceRewindv",
	"alGetString",
	"alBufferData",
	"alIsSource",
	"alGetInteger",
	"alGetSourcei",
	"alSourceQueueBuffers",
	"alListenerf",
	"alGenBuffers",
	"alcGetProcAddress",
	"alGetBufferf",
	"alGetListeneri",
	"alGetDoublev",
	"alcCreateContext",
	"alGetBoolean",
	"alGetListenerf",
];

pub unsafe fn load_al_funcs() {
	for func in FUNCS {
		load_al_func(func);
	}
}

unsafe fn load_al_func(func: &str) {
	let func_str = CString::new(func).unwrap();

	let openal_module_name = CString::new("libopenal.so").unwrap();
	let openal_module = dlopen(openal_module_name.as_ptr(), RTLD_LAZY);
	if openal_module.is_null() {
		let error = dlerror();
		let error = CStr::from_ptr(error).to_str().unwrap();
		panic!("{}", error);
	}
	let real_func = dlsym(openal_module, func_str.as_ptr());
	if real_func.is_null() {
		panic!("{func} not found in libopenal");
	}

	let module = dlopen(std::ptr::null(), RTLD_LAZY);
	let func_ptr = dlsym(module, func_str.as_ptr());
	if func_ptr.is_null() {
		panic!("{func} not found in main");
	}
	assert_ne!(func_ptr, real_func);

	let real_func = real_func as usize;
	let real_func = real_func.to_le_bytes();
	let mut data = Vec::with_capacity(7);
	data.push(0xB8);
	for i in real_func {
		data.push(i);
	}
	data.push(0xFF);
	data.push(0xE0);

	hook::write_memory(func_ptr as *mut (), &data);
}
