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

	gl::load_gl_funcs(&device.glfw);

	BIND_FRAMEBUFFER = Some(transmute(
		device.glfw.get_proc_address_raw("glBindFramebuffer"),
	));
	BLIT_FRAMEBUFFER = Some(transmute(
		device.glfw.get_proc_address_raw("glBlitFramebuffer"),
	));
	CLEAR_BUFFER = Some(transmute(
		device.glfw.get_proc_address_raw("glClearBufferfv"),
	));

	let mut fbo = 0;
	let mut texture = 0;

	let bind_framebuffer = BIND_FRAMEBUFFER.unwrap();
	let gen_framebuffer: extern "C" fn(i32, *mut u32) =
		transmute(device.glfw.get_proc_address_raw("glGenFramebuffers"));
	let gen_texture: extern "C" fn(i32, *mut u32) =
		transmute(device.glfw.get_proc_address_raw("glGenTextures"));
	let bind_texture: extern "C" fn(i32, u32) =
		transmute(device.glfw.get_proc_address_raw("glBindTexture"));
	let tex_image: extern "C" fn(i32, i32, i32, u32, u32, i32, i32, i32, *const c_void) =
		transmute(device.glfw.get_proc_address_raw("glTexImage2D"));
	let framebuffer_texture: extern "C" fn(i32, i32, i32, u32, i32) =
		transmute(device.glfw.get_proc_address_raw("glFramebufferTexture2D"));

	gen_framebuffer(1, &mut fbo);
	bind_framebuffer(GL_FRAMEBUFFER, fbo);

	gen_texture(1, &mut texture);
	bind_texture(GL_TEXTURE_2D, texture);
	tex_image(
		GL_TEXTURE_2D,
		0,
		GL_RGB,
		CONFIG.width,
		CONFIG.height,
		0,
		GL_RGB,
		GL_UNSIGNED_BYTE,
		std::ptr::null(),
	);

	framebuffer_texture(
		GL_FRAMEBUFFER,
		GL_COLOR_ATTACHMENT0,
		GL_TEXTURE_2D,
		texture,
		0,
	);
	bind_framebuffer(GL_FRAMEBUFFER, 0);
	bind_texture(GL_TEXTURE_2D, 0);

	let adm = AdmWindow {
		ident: [b'W', b'N', b'D', b'W'],
		window,
		fbo,
	};

	Box::leak(Box::new(adm))
}

static mut BIND_FRAMEBUFFER: Option<unsafe extern "C" fn(i32, u32)> = None;
static mut BLIT_FRAMEBUFFER: Option<
	unsafe extern "C" fn(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32),
> = None;
static mut CLEAR_BUFFER: Option<unsafe extern "C" fn(i32, i32, *const f32)> = None;

const GL_TEXTURE_2D: i32 = 0x0DE1;
const GL_UNSIGNED_BYTE: i32 = 0x1401;
const GL_COLOR: i32 = 0x1800;
const GL_RGB: i32 = 0x1907;
const GL_NEAREST: i32 = 0x2600;
const GL_COLOR_BUFFER_BIT: i32 = 0x4000;
const GL_READ_FRAMEBUFFER: i32 = 0x8CA8;
const GL_DRAW_FRAMEBUFFER: i32 = 0x8CA9;
const GL_COLOR_ATTACHMENT0: i32 = 0x8CE0;
const GL_FRAMEBUFFER: i32 = 0x8D40;

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

	let bind = BIND_FRAMEBUFFER.unwrap();
	let blit = BLIT_FRAMEBUFFER.unwrap();
	let clear = CLEAR_BUFFER.unwrap();

	bind(GL_READ_FRAMEBUFFER, 0);
	bind(GL_DRAW_FRAMEBUFFER, window.fbo);
	blit(
		0,
		0,
		CONFIG.width as i32,
		CONFIG.height as i32,
		0,
		0,
		CONFIG.width as i32,
		CONFIG.height as i32,
		GL_COLOR_BUFFER_BIT,
		GL_NEAREST,
	);

	bind(GL_FRAMEBUFFER, 0);
	let clear_color = [0.0, 0.0, 0.0, 1.0];
	clear(GL_COLOR, 0, clear_color.as_ptr());

	bind(GL_READ_FRAMEBUFFER, window.fbo);
	bind(GL_DRAW_FRAMEBUFFER, 0);
	blit(
		0,
		0,
		CONFIG.width as i32,
		CONFIG.height as i32,
		viewport_x,
		viewport_y,
		viewport_x + viewport_width,
		viewport_y + viewport_height,
		GL_COLOR_BUFFER_BIT,
		GL_NEAREST,
	);

	bind(GL_FRAMEBUFFER, 0);

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
