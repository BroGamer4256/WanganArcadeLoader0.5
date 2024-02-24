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
	block_sudo: bool,
	deadzone: f32,
	width: i32,
	height: i32,
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

	let block_sudo = if let Some(config) = CONFIG.as_ref() {
		config.block_sudo
	} else {
		true
	};

	if !block_sudo || str.starts_with("find") {
		let command = str.replace("/tmp/", "./tmp/");
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
	let filename = if filename.starts_with("/tmp") {
		CString::new(filename.replace("/tmp/", "./tmp/")).unwrap()
	} else {
		CString::new(filename).unwrap()
	};

	let fopen = CString::new("fopen").unwrap();
	let fopen = dlsym(RTLD_NEXT, fopen.as_ptr());
	let fopen: extern "C" fn(*const c_char, *const c_char) -> *const () = transmute(fopen);
	fopen(filename.as_ptr(), mode)
}

#[no_mangle]
unsafe extern "C" fn rename(old: *const c_char, new: *const c_char) -> c_int {
	let old = CStr::from_ptr(old).to_str().unwrap();
	let old = old.replace("/tmp/", "./tmp/");
	let old = CString::new(old).unwrap();

	let new = CStr::from_ptr(new).to_str().unwrap();
	let new = new.replace("/tmp/", "./tmp/");
	let new = CString::new(new).unwrap();

	let rename = CString::new("rename").unwrap();
	let rename = dlsym(RTLD_NEXT, rename.as_ptr());
	let rename: extern "C" fn(*const c_char, *const c_char) -> c_int = transmute(rename);
	rename(old.as_ptr(), new.as_ptr())
}

#[no_mangle]
unsafe extern "C" fn _ZNSt13basic_filebufIcSt11char_traitsIcEE4openEPKcSt13_Ios_Openmode(
	this: c_int,
	filename: *const c_char,
	mode: c_int,
) -> *const () {
	if let Ok(filename) = CStr::from_ptr(filename).to_str() {
		let filename = if filename.starts_with("/tmp") {
			CString::new(filename.replace("/tmp/", "./tmp/")).unwrap()
		} else {
			CString::new(filename).unwrap()
		};

		let open =
			CString::new("_ZNSt13basic_filebufIcSt11char_traitsIcEE4openEPKcSt13_Ios_Openmode")
				.unwrap();
		let open = dlsym(RTLD_NEXT, open.as_ptr());
		let open: extern "C" fn(c_int, *const c_char, c_int) -> *const () = transmute(open);
		open(this, filename.as_ptr(), mode)
	} else {
		let open =
			CString::new("_ZNSt13basic_filebufIcSt11char_traitsIcEE4openEPKcSt13_Ios_Openmode")
				.unwrap();
		let open = dlsym(RTLD_NEXT, open.as_ptr());
		let open: extern "C" fn(c_int, *const c_char, c_int) -> *const () = transmute(open);
		open(this, filename, mode)
	}
}

static mut LUA_GETGLOBAL: Option<extern "C" fn(*const c_void, *const c_char) -> c_int> = None;
static mut LUA_SETGLOBAL: Option<extern "C" fn(*const c_void, *const c_char)> = None;
static mut LUA_PUSHNUMBER: Option<extern "C" fn(*const c_void, c_double)> = None;
unsafe extern "C" fn lua_getglobal(state: *const c_void, global: *const c_char) -> c_int {
	if let Some(config) = CONFIG.as_ref() {
		let str = CStr::from_ptr(global).to_str().unwrap();
		match str {
			"SCREEN_XSIZE" => {
				LUA_PUSHNUMBER.unwrap()(state, config.width as c_double);
				LUA_SETGLOBAL.unwrap()(state, global);
			}
			"SCREEN_YSIZE" => {
				LUA_PUSHNUMBER.unwrap()(state, config.height as c_double);
				LUA_SETGLOBAL.unwrap()(state, global);
			}
			"MINIMAP_DISP_X" => {
				LUA_PUSHNUMBER.unwrap()(state, (config.width as c_double * 0.0265625).round());
				LUA_SETGLOBAL.unwrap()(state, global);
			}
			"MINIMAP_DISP_Y" => {
				LUA_PUSHNUMBER.unwrap()(state, (config.height as c_double * 0.2364).round());
				LUA_SETGLOBAL.unwrap()(state, global);
			}
			_ => {}
		};
	}

	LUA_GETGLOBAL.unwrap()(state, global)
}

static mut ORIGINAL_SET_VIEWPORT: Option<
	extern "C" fn(*const c_void, c_int, c_int, c_int, c_int, c_float, c_float) -> *const c_void,
> = None;
unsafe extern "C" fn set_viewport(
	this: *const c_void,
	a1: c_int,
	a2: c_int,
	width: c_int,
	height: c_int,
	a5: c_float,
	a6: c_float,
) -> *const c_void {
	if width == 88 && height == 82 {
		if let Some(config) = CONFIG.as_ref() {
			let width = config.width as f32 * 0.1375;
			let height = config.height as f32 * 0.17;
			return ORIGINAL_SET_VIEWPORT.unwrap()(
				this,
				a1,
				a2,
				width as c_int,
				height as c_int,
				a5,
				a6,
			);
		}
	}
	ORIGINAL_SET_VIEWPORT.unwrap()(this, a1, a2, width, height, a5, a6)
}

static mut ORIGINAL_MAKE_PERSPECTIVE: Option<
	extern "C" fn(*const c_void, c_float, c_float, c_float, c_float, c_float),
> = None;
unsafe extern "C" fn make_perspective(
	this: *const c_void,
	fov: c_float,
	a2: c_float,
	aspect_ratio: c_float,
	a4: c_float,
	a5: c_float,
) {
	if let Some(config) = CONFIG.as_ref() {
		let width = config.width as f32;
		let height = config.height as f32;
		let aspect_ratio = if aspect_ratio == 640.0 / 480.0 {
			width / height
		} else {
			aspect_ratio
		};
		let fov_ratio = fov / (640.0 / 480.0);
		let fov = fov_ratio * aspect_ratio;
		return ORIGINAL_MAKE_PERSPECTIVE.unwrap()(this, fov, a2, aspect_ratio, a4, a5);
	}
	ORIGINAL_MAKE_PERSPECTIVE.unwrap()(this, fov, a2, aspect_ratio, a4, a5)
}

#[ctor::ctor]
unsafe fn init() {
	let exe = std::env::current_exe().unwrap();
	if !exe.ends_with("main") {
		panic!("Not 3DX+");
	}

	if let Ok(toml) = std::fs::read_to_string("config.toml") {
		CONFIG = toml::from_str(&toml).ok();
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

	LUA_GETGLOBAL = Some(transmute(hook::hook_symbol(
		"lua_getglobal",
		lua_getglobal as *const (),
	)));
	LUA_SETGLOBAL = Some(transmute(hook::get_symbol("lua_setglobal")));
	LUA_PUSHNUMBER = Some(transmute(hook::get_symbol("lua_pushnumber")));
	ORIGINAL_SET_VIEWPORT = Some(transmute(hook::hook_symbol(
		"_ZN3Gap3Gfx19igAGLEVisualContext11setViewportEiiiiff",
		set_viewport as *const (),
	)));
	ORIGINAL_MAKE_PERSPECTIVE = Some(transmute(hook::hook_symbol(
		"_ZN3Gap4Math11igMatrix44f32makePerspectiveProjectionRadiansEfffff",
		make_perspective as *const (),
	)));

	adm::init();
	al::load_al_funcs();

	if let Some(config) = CONFIG.as_ref() {
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
