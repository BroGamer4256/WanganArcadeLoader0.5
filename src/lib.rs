use libc::*;
use poll::*;
use std::ffi::{CStr, CString};
use std::mem::transmute;

pub mod adm;
pub mod al;
pub mod card;
pub mod gl;
pub mod hook;
pub mod jamma;
pub mod poll;

#[derive(serde::Deserialize)]
pub struct Config {
	fullscreen: bool,
	input_emu: bool,
	card_emu: bool,
	deadzone: f32,
}

pub struct KeyConfig {
	test: KeyBindings,
	service: KeyBindings,
	quit: KeyBindings,
	card_insert: KeyBindings,

	gear_next: KeyBindings,
	gear_previous: KeyBindings,
	gear_neutral: KeyBindings,
	gear_first: KeyBindings,
	gear_second: KeyBindings,
	gear_third: KeyBindings,
	gear_fourth: KeyBindings,
	gear_fifth: KeyBindings,
	gear_sixth: KeyBindings,

	perspective: KeyBindings,
	intrude: KeyBindings,
	gas: KeyBindings,
	brake: KeyBindings,
	wheel_left: KeyBindings,
	wheel_right: KeyBindings,
}

pub static mut CONFIG: Option<Config> = None;
pub static mut KEYCONFIG: Option<KeyConfig> = None;

pub extern "C" fn adachi() -> c_int {
	true as c_int
}

#[no_mangle]
unsafe extern "C" fn system(command: *const c_char) -> c_int {
	let cstr = CStr::from_ptr(command);
	let str = cstr.to_str().unwrap();
	if str.starts_with("find") {
		let command = str.replace("/tmp/data/", "./tmp/data/");
		let command = CString::new(command).unwrap();

		let system = CString::new("system").unwrap();
		let system = dlsym(RTLD_NEXT, system.as_ptr());
		let system: extern "C" fn(*const c_char) -> c_int = transmute(system);

		system(command.as_ptr())
	} else {
		dbg!(str);
		0
	}
}

#[no_mangle]
unsafe extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *const () {
	let filename = CStr::from_ptr(filename).to_str().unwrap();
	let filename = filename.replace("/tmp/", "./tmp/");
	let filename = CString::new(filename).unwrap();

	let fopen = CString::new("fopen").unwrap();
	let fopen = dlsym(RTLD_NEXT, fopen.as_ptr());
	let fopen: extern "C" fn(*const c_char, *const c_char) -> *const () = transmute(fopen);
	fopen(filename.as_ptr(), mode)
}

#[no_mangle]
unsafe extern "C" fn _ZNSt13basic_filebufIcSt11char_traitsIcEE4openEPKcSt13_Ios_Openmode(
	_test: c_int,
	filename: *const c_char,
	mode: c_int,
) -> *const () {
	if let Ok(filename) = CStr::from_ptr(filename).to_str() {
		let filename = filename.replace("/tmp/", "./tmp/");
		let filename = CString::new(filename).unwrap();

		let open =
			CString::new("_ZNSt13basic_filebufIcSt11char_traitsIcEE4openEPKcSt13_Ios_Openmode")
				.unwrap();
		let open = dlsym(RTLD_NEXT, open.as_ptr());
		let open: extern "C" fn(c_int, *const c_char, c_int) -> *const () = transmute(open);
		open(_test, filename.as_ptr(), mode)
	} else {
		let open =
			CString::new("_ZNSt13basic_filebufIcSt11char_traitsIcEE4openEPKcSt13_Ios_Openmode")
				.unwrap();
		let open = dlsym(RTLD_NEXT, open.as_ptr());
		let open: extern "C" fn(c_int, *const c_char, c_int) -> *const () = transmute(open);
		open(_test, filename, mode)
	}
}

#[ctor::ctor]
unsafe fn init() {
	let exe = std::env::current_exe().unwrap();
	if !exe.ends_with("main") {
		panic!("Not 3DX+");
	}

	if let Ok(toml) = std::fs::read_to_string("config.toml") {
		CONFIG = Some(toml::from_str(&toml).unwrap());
	}

	// Really what I should do is implement a custom serde::Deserialize for KeyBindings
	// but serdes documentation is really confusing when it comes to this
	#[derive(serde::Deserialize)]
	#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
	struct KeyConfigTemp {
		test: Vec<String>,
		service: Vec<String>,
		quit: Vec<String>,
		card_insert: Vec<String>,

		gear_next: Vec<String>,
		gear_previous: Vec<String>,
		gear_neutral: Vec<String>,
		gear_first: Vec<String>,
		gear_second: Vec<String>,
		gear_third: Vec<String>,
		gear_fourth: Vec<String>,
		gear_fifth: Vec<String>,
		gear_sixth: Vec<String>,

		perspective: Vec<String>,
		intrude: Vec<String>,
		gas: Vec<String>,
		brake: Vec<String>,
		wheel_left: Vec<String>,
		wheel_right: Vec<String>,
	}

	let toml = std::fs::read_to_string("keyconfig.toml").unwrap();
	let keyconfig: KeyConfigTemp = toml::from_str(&toml).unwrap();
	let keyconfig = KeyConfig {
		test: parse_keybinding(keyconfig.test),
		service: parse_keybinding(keyconfig.service),
		quit: parse_keybinding(keyconfig.quit),
		card_insert: parse_keybinding(keyconfig.card_insert),

		gear_next: parse_keybinding(keyconfig.gear_next),
		gear_previous: parse_keybinding(keyconfig.gear_previous),
		gear_neutral: parse_keybinding(keyconfig.gear_neutral),
		gear_first: parse_keybinding(keyconfig.gear_first),
		gear_second: parse_keybinding(keyconfig.gear_second),
		gear_third: parse_keybinding(keyconfig.gear_third),
		gear_fourth: parse_keybinding(keyconfig.gear_fourth),
		gear_fifth: parse_keybinding(keyconfig.gear_fifth),
		gear_sixth: parse_keybinding(keyconfig.gear_sixth),

		perspective: parse_keybinding(keyconfig.perspective),
		intrude: parse_keybinding(keyconfig.intrude),
		gas: parse_keybinding(keyconfig.gas),
		brake: parse_keybinding(keyconfig.brake),
		wheel_left: parse_keybinding(keyconfig.wheel_left),
		wheel_right: parse_keybinding(keyconfig.wheel_right),
	};
	KEYCONFIG = Some(keyconfig);

	hook::hook_symbol("_ZNK6clHaspcvbEv", adachi as *const ());
	hook::hook_symbol("_ZNK7clHasp2cvbEv", adachi as *const ());
	hook::hook_symbol("_ZN18clSeqBootNetThread3runEPv", adachi as *const ());
	adm::init();
	al::load_al_funcs();

	if let Some(config) = &CONFIG {
		if config.input_emu {
			jamma::init();
		}
		if config.card_emu {
			card::init();
		}
	} else {
		jamma::init();
		card::init();
	}
}
