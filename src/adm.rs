use crate::*;
use glfw::*;
use std::mem::forget;

extern "C" fn adm_version() -> *const c_char {
	let cstr = CString::new("WanganArcadeLoader 0.1").unwrap();
	let ptr = cstr.as_ptr();
	forget(cstr);
	ptr
}

pub static mut WINDOW_HANDLE: Option<*mut c_void> = None;

#[repr(C)]
struct AdmDevice {
	ident: [u8; 4], // DEVI
	glfw: Glfw,
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Default)]
struct AdmChooseMode {
	ident: [u8; 4], // MOCF
	unk_0x04: u32,
	unk_0x08: u32,
	unk_0x0C: u32,
	unk_0x10: u32,
	unk_0x14: u32,
	width: u32,
	height: u32,
	refresh: u32,
}

#[repr(C)]
struct AdmWindow {
	ident: [u8; 4], // WNDW
	window: PWindow,
	fbo: u32,
}

extern "C" fn adm_device() -> *const AdmDevice {
	let glfw = glfw::init(glfw::fail_on_errors).unwrap();
	let adm = AdmDevice {
		ident: [b'D', b'E', b'V', b'I'],
		glfw,
	};
	Box::leak(Box::new(adm))
}

extern "C" fn adm_config() -> *const *const AdmChooseMode {
	let adm = AdmChooseMode {
		ident: [b'M', b'O', b'C', b'F'],
		refresh: 60,
		..Default::default()
	};
	Box::leak(Box::new(Box::leak(Box::new(adm)) as *const AdmChooseMode))
}

extern "C" fn adm_fb_config() -> *const u8 {
	Box::leak(Box::new(0))
}

unsafe extern "C" fn adm_window(device: *mut AdmDevice) -> *const AdmWindow {
	let device = device.as_mut().unwrap();
	let monitor = Monitor::from_primary();
	let window_mode = if CONFIG.fullscreen {
		WindowMode::FullScreen(&monitor)
	} else {
		WindowMode::Windowed
	};
	device.glfw.window_hint(WindowHint::Resizable(false)); // Force floating on tiling window managers
	let (mut window, _) = device
		.glfw
		.create_window(
			CONFIG.width,
			CONFIG.height,
			"WanganArcadeLoader",
			window_mode,
		)
		.unwrap();
	WINDOW_HANDLE = Some(window.get_x11_window());
	window.make_current();
	window.set_resizable(true);
	device.glfw.set_swap_interval(SwapInterval::Sync(1));

	opengl::load_gl_funcs(&device.glfw);
	gl::load_with(|s| device.glfw.get_proc_address_raw(s));

	let mut fbo = 0;
	let mut texture = 0;
	gl::GenFramebuffers(1, &mut fbo);
	gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

	gl::GenTextures(1, &mut texture);
	gl::BindTexture(gl::TEXTURE_2D, texture);
	gl::TexImage2D(
		gl::TEXTURE_2D,
		0,
		gl::RGB as i32,
		CONFIG.width as i32,
		CONFIG.height as i32,
		0,
		gl::RGB,
		gl::UNSIGNED_BYTE,
		std::ptr::null(),
	);
	gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
	gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
	gl::FramebufferTexture2D(
		gl::FRAMEBUFFER,
		gl::COLOR_ATTACHMENT0,
		gl::TEXTURE_2D,
		texture,
		0,
	);
	gl::BindTexture(gl::TEXTURE_2D, 0);
	gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

	let adm = AdmWindow {
		ident: [b'W', b'N', b'D', b'W'],
		window,
		fbo,
	};

	Box::leak(Box::new(adm))
}

unsafe extern "C" fn adm_swap_buffers(window: *mut AdmWindow) -> c_int {
	let window = window.as_mut().unwrap();
	let (window_width, window_height) = window.window.get_size();
	let window_ar = window_width as f32 / window_height as f32;
	let ar = CONFIG.width as f32 / CONFIG.height as f32;

	let (viewport_width, viewport_height, viewport_x, viewport_y) = if window_ar > ar {
		let viewport_width: i32 = ((window_height as f32) * ar) as i32;
		let viewport_x = ((window_width - viewport_width) as f32 / 2.0) as i32;
		(viewport_width, window_height, viewport_x, 0)
	} else {
		let viewport_height = ((window_width as f32) / ar) as i32;
		let viewport_y = ((window_height - viewport_height) as f32 / 2.0) as i32;
		(window_width, viewport_height, 0, viewport_y)
	};



	// Upscaling + black bars
	gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
	gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, window.fbo);
	gl::BlitFramebuffer(
		0,
		0,
		CONFIG.width as i32,
		CONFIG.height as i32,
		0,
		0,
		CONFIG.width as i32,
		CONFIG.height as i32,
		gl::COLOR_BUFFER_BIT,
		gl::NEAREST,
	);

	gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
	gl::Clear(gl::COLOR_BUFFER_BIT);

	gl::BindFramebuffer(gl::READ_FRAMEBUFFER, window.fbo);
	gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
	gl::BlitFramebuffer(
		0,
		0,
		CONFIG.width as i32,
		CONFIG.height as i32,
		viewport_x,
		viewport_y,
		viewport_x + viewport_width,
		viewport_y + viewport_height,
		gl::COLOR_BUFFER_BIT,
		gl::NEAREST,
	);

	gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

	window.window.swap_buffers();

	0
}

static mut CL_APP_INSTANCE: Option<extern "C" fn() -> *const c_void> = None;
static mut CL_APP_IS_MAIN_THREAD: Option<extern "C" fn(*const c_void) -> bool> = None;
static mut ORIGINAL_DEL_SPRITE_MANAGER: Option<extern "C" fn(*const c_void)> = None;
unsafe extern "C" fn del_sprite_manager(this: *const c_void) {
	let cl_app = CL_APP_INSTANCE.unwrap()();
	if CL_APP_IS_MAIN_THREAD.unwrap()(cl_app) {
		ORIGINAL_DEL_SPRITE_MANAGER.unwrap()(this);
	}
}

pub unsafe fn init() {
	hook::hook_symbol("admvt_setup", adachi as *const ());
	hook::hook_symbol("admShutdown", adachi as *const ());
	hook::hook_symbol("admGetString", adm_version as *const ());
	hook::hook_symbol("admGetNumDevices", adachi as *const ());
	hook::hook_symbol("admInitDevicei", adm_device as *const ());
	hook::hook_symbol("admChooseModeConfigi", adm_config as *const ());
	hook::hook_symbol("admModeConfigi", adachi as *const ());
	hook::hook_symbol("admChooseFBConfigi", adm_fb_config as *const ());
	hook::hook_symbol("admCreateScreeni", adachi as *const ());
	hook::hook_symbol("admCreateGraphicsContext", adachi as *const ());
	hook::hook_symbol("admCreateWindowi", adm_window as *const ());
	hook::hook_symbol("admDisplayScreen", adachi as *const ());
	hook::hook_symbol("admMakeContextCurrent", adachi as *const ());
	hook::hook_symbol("admSwapInterval", adachi as *const ());
	hook::hook_symbol("admCursorAttribi", adachi as *const ());
	hook::hook_symbol("admGetDeviceAttribi", adachi as *const ());
	hook::hook_symbol("admSwapBuffers", adm_swap_buffers as *const ());
	hook::hook_symbol("admSetMonitorGamma", adachi as *const ());
	ORIGINAL_DEL_SPRITE_MANAGER = Some(transmute(hook::hook_symbol(
		"_ZN15clSpriteManagerD1Ev",
		del_sprite_manager as *const (),
	)));
	CL_APP_INSTANCE = Some(transmute(hook::get_symbol(
		"_ZN11clAppSystem11getInstanceEv",
	)));
	CL_APP_IS_MAIN_THREAD = Some(transmute(hook::get_symbol(
		"_ZN11clAppSystem12isMainThreadEv",
	)));
}
