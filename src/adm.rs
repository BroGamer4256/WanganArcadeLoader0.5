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
	glfw: Glfw,
	window: PWindow,
	fbo: u32,
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

unsafe extern "C" fn adm_window() -> *mut AdmWindow {
	let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
	glfw.window_hint(WindowHint::Resizable(false)); // Force floating on tiling window managers
	let (mut window, _) = glfw.with_primary_monitor(|glfw, m| {
		let window_mode = if CONFIG.fullscreen && m.is_some() {
			WindowMode::FullScreen(m.unwrap())
		} else {
			WindowMode::Windowed
		};
		glfw.create_window(
			CONFIG.width,
			CONFIG.height,
			"WanganArcadeLoader",
			window_mode,
		)
		.unwrap()
	});
	WINDOW_HANDLE = Some(window.get_x11_window());
	window.make_current();
	window.set_resizable(true);
	glfw.set_swap_interval(SwapInterval::Sync(1));

	opengl::load_gl_funcs(&glfw);
	gl::load_with(|s| glfw.get_proc_address_raw(s));

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
		glfw,
		window,
		fbo,
	};

	Box::leak(Box::new(adm))
}

unsafe extern "C" fn adm_swap_buffers(window_ptr: *mut AdmWindow) -> c_int {
	let window = window_ptr.as_mut().unwrap();

	let graphics =
		hook::get_symbol("_ZN11teSingletonI10clGraphicsE11sm_instanceE") as *mut *mut u16;
	let graphics = graphics.read();

	// Upscaling + black bars only if the game isnt using a saved frame
	let should_blit = if GAME_VERSION.is_wm3() {
		graphics.byte_offset(0x48).read() != 0
	} else {
		if graphics.byte_offset(0x54).read() != 0 {
			true
		} else {
			let graphics = graphics as *mut *mut u32;
			let buffer = graphics.byte_offset(0x0C).read();
			let buffer = if buffer.is_null() {
				graphics.byte_offset(0x08).read()
			} else {
				buffer
			};

			// If frame is saved dont blit
			buffer.byte_offset(0x04).read() == 0
		}
	};

	if should_blit {
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
	}

	window.window.swap_buffers();
	window.glfw.poll_events();
	if window.window.should_close() {
		let window = window_ptr.read().window;
		drop(window);
		exit(0);
	}

	0
}

static mut CL_APP_INSTANCE: Option<extern "C" fn() -> *const c_void> = None;
static mut CL_APP_IS_MAIN_THREAD: Option<extern "C" fn(*const c_void) -> bool> = None;
static mut CL_MAIN_INSTANCE: *const *const *const c_void = std::ptr::null();
static mut THREAD_MANAGER_CURRENT: Option<extern "C" fn(*const c_void) -> *const c_void> = None;
static mut CALL_FROM_MAIN_THREAD: Option<
	extern "C" fn(*const c_void, *const fn(*const c_void), *const c_void),
> = None;

static mut ORIGINAL_DEL_SPRITE_MANAGER: Option<extern "C" fn(*const c_void)> = None;
unsafe extern "C" fn del_sprite_manager(this: *const c_void) {
	let cl_app = CL_APP_INSTANCE.unwrap()();
	if CL_APP_IS_MAIN_THREAD.unwrap()(cl_app) {
		ORIGINAL_DEL_SPRITE_MANAGER.unwrap()(this);
	} else {
		let thread_manager = CL_MAIN_INSTANCE.read().byte_offset(0x40).read();
		let current = THREAD_MANAGER_CURRENT.unwrap()(thread_manager);
		CALL_FROM_MAIN_THREAD.unwrap()(current, del_sprite_manager as *const _, this);
	}
}

static mut ORIGINAL_SAVE_IMAGE: Option<extern "C" fn(*const c_void, *const c_void)> = None;
unsafe extern "C" fn save_image(render_buffer: *const c_void, filepath: *const c_void) {
	let cl_app = CL_APP_INSTANCE.unwrap()();
	if CL_APP_IS_MAIN_THREAD.unwrap()(cl_app) {
		ORIGINAL_SAVE_IMAGE.unwrap()(render_buffer, filepath);
	} else {
		let args = Box::new((render_buffer, filepath));
		let thread_manager = CL_MAIN_INSTANCE.read().byte_offset(0x40).read();
		let current = THREAD_MANAGER_CURRENT.unwrap()(thread_manager);
		CALL_FROM_MAIN_THREAD.unwrap()(
			current,
			save_image_main as *const _,
			transmute(args.as_ref()),
		);
	}
}

unsafe extern "C" fn save_image_main(args: *const c_void) {
	let args: &(*const c_void, *const c_void) = transmute(args);
	let (render_buffer, filepath) = *args;
	let cl_app = CL_APP_INSTANCE.unwrap()();
	if CL_APP_IS_MAIN_THREAD.unwrap()(cl_app) {
		ORIGINAL_SAVE_IMAGE.unwrap()(render_buffer, filepath);
	} else {
		panic!("Not main thread!");
	}
}

static mut ORIGINAL_CREATE_TEXTURE_HANDLE: Option<extern "C" fn(*const c_void, i32, i32) -> i32> =
	None;
unsafe extern "C" fn create_texture_handle(this: *const c_void, a1: i32, a2: i32) -> i32 {
	let cl_app = CL_APP_INSTANCE.unwrap()();
	if CL_APP_IS_MAIN_THREAD.unwrap()(cl_app) {
		ORIGINAL_CREATE_TEXTURE_HANDLE.unwrap()(this, a1, a2)
	} else {
		let args = Box::new((this, a1, a2));
		let thread_manager = CL_MAIN_INSTANCE.read().byte_offset(0x40).read();
		let current = THREAD_MANAGER_CURRENT.unwrap()(thread_manager);
		CALL_FROM_MAIN_THREAD.unwrap()(
			current,
			create_texture_handle_main as *const _,
			transmute(args.as_ref()),
		);
		1
	}
}

unsafe extern "C" fn create_texture_handle_main(args: *const c_void) {
	let args: &(*const c_void, i32, i32) = transmute(args);
	let (this, a1, a2) = *args;
	let cl_app = CL_APP_INSTANCE.unwrap()();
	if CL_APP_IS_MAIN_THREAD.unwrap()(cl_app) {
		ORIGINAL_CREATE_TEXTURE_HANDLE.unwrap()(this, a1, a2);
	} else {
		panic!("Not main thread!");
	}
}

static mut ORIGINAL_SET_TEXTURE: Option<extern "C" fn(*const c_void, i32, i32) -> i32> = None;
unsafe extern "C" fn set_texture(this: *const c_void, a1: i32, a2: i32) -> i32 {
	let cl_app = CL_APP_INSTANCE.unwrap()();
	if CL_APP_IS_MAIN_THREAD.unwrap()(cl_app) {
		ORIGINAL_SET_TEXTURE.unwrap()(this, a1, a2)
	} else {
		let args = Box::new((this, a1, a2));
		let thread_manager = CL_MAIN_INSTANCE.read().byte_offset(0x40).read();
		let current = THREAD_MANAGER_CURRENT.unwrap()(thread_manager);
		CALL_FROM_MAIN_THREAD.unwrap()(
			current,
			set_texture_main as *const _,
			transmute(args.as_ref()),
		);
		1
	}
}

unsafe extern "C" fn set_texture_main(args: *const c_void) {
	let args: &(*const c_void, i32, i32) = transmute(args);
	let (this, a1, a2) = *args;
	let cl_app = CL_APP_INSTANCE.unwrap()();
	if CL_APP_IS_MAIN_THREAD.unwrap()(cl_app) {
		ORIGINAL_SET_TEXTURE.unwrap()(this, a1, a2);
	} else {
		panic!("Not main thread!");
	}
}

static mut ORIGINAL_SET_TEXTURE_REGION: Option<
	extern "C" fn(*const c_void, i32, i32, i32, i32, i32, i32, *const c_void) -> i32,
> = None;
unsafe extern "C" fn set_texture_region(
	this: *const c_void,
	a1: i32,
	a2: i32,
	a3: i32,
	a4: i32,
	a5: i32,
	a6: i32,
	a7: *const c_void,
) -> i32 {
	let cl_app = CL_APP_INSTANCE.unwrap()();
	if CL_APP_IS_MAIN_THREAD.unwrap()(cl_app) {
		ORIGINAL_SET_TEXTURE_REGION.unwrap()(this, a1, a2, a3, a4, a5, a6, a7)
	} else {
		let args = Box::new((this, a1, a2, a3, a4, a5, a6, a7));
		let thread_manager = CL_MAIN_INSTANCE.read().byte_offset(0x40).read();
		let current = THREAD_MANAGER_CURRENT.unwrap()(thread_manager);
		CALL_FROM_MAIN_THREAD.unwrap()(
			current,
			set_texture_region_main as *const _,
			transmute(args.as_ref()),
		);
		1
	}
}

unsafe extern "C" fn set_texture_region_main(args: *const c_void) {
	let args: &(*const c_void, i32, i32, i32, i32, i32, i32, *const c_void) = transmute(args);
	let (this, a1, a2, a3, a4, a5, a6, a7) = *args;
	let cl_app = CL_APP_INSTANCE.unwrap()();
	if CL_APP_IS_MAIN_THREAD.unwrap()(cl_app) {
		ORIGINAL_SET_TEXTURE_REGION.unwrap()(this, a1, a2, a3, a4, a5, a6, a7);
	} else {
		panic!("Not main thread!");
	}
}

pub unsafe fn init() {
	hook::hook_symbol("admvt_setup", adachi as *const ());
	hook::hook_symbol("admShutdown", adachi as *const ());
	hook::hook_symbol("admGetString", adm_version as *const ());
	hook::hook_symbol("admGetNumDevices", adachi as *const ());
	hook::hook_symbol("admInitDevicei", adachi as *const ());
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

	CL_APP_INSTANCE = Some(transmute(hook::get_symbol(
		"_ZN11clAppSystem11getInstanceEv",
	)));
	CL_APP_IS_MAIN_THREAD = Some(transmute(hook::get_symbol(
		"_ZN11clAppSystem12isMainThreadEv",
	)));
	CL_MAIN_INSTANCE = transmute(hook::get_symbol(
		"_ZN11teSingletonI10teSequenceI6clMainEE11sm_instanceE",
	));
	THREAD_MANAGER_CURRENT = Some(transmute(hook::get_symbol(
		"_ZN17clNPThreadManager7currentEv",
	)));
	CALL_FROM_MAIN_THREAD = transmute(hook::get_symbol(
		"_ZN10clNPThread26callFunctionFromMainThreadEPFvPvES0_",
	));

	ORIGINAL_DEL_SPRITE_MANAGER = Some(transmute(hook::hook_symbol(
		"_ZN15clSpriteManagerD1Ev",
		del_sprite_manager as *const (),
	)));
	ORIGINAL_SAVE_IMAGE = Some(transmute(hook::hook_symbol(
		"_ZN14clRenderBuffer9saveImageEPKc",
		save_image as *const (),
	)));
	ORIGINAL_CREATE_TEXTURE_HANDLE = Some(transmute(hook::hook_symbol(
		"_ZN24clAlchemyTextureAccessor19createTextureHandleEii",
		create_texture_handle as *const (),
	)));
	ORIGINAL_SET_TEXTURE = Some(transmute(hook::hook_symbol(
		"_ZN3Gap3Gfx19igAGLEVisualContext10setTextureEii",
		set_texture as *const (),
	)));
	ORIGINAL_SET_TEXTURE_REGION = Some(transmute(hook::hook_symbol(
		"_ZN3Gap3Gfx19igAGLEVisualContext16setTextureRegionEiiiiiiPNS0_7igImageE",
		set_texture_region as *const (),
	)));
}
