use crate::*;

static mut SDL_STATE: Option<poll::PollState> = None;

static mut GEAR_INDEX: u8 = 0;

fn set_gear_bits(index: u8) -> u32 {
	const LEFT: u32 = 1;
	const RIGHT: u32 = 2;
	const TOP: u32 = 4;
	const BOTTOM: u32 = 8;
	match index {
		0 => 0,
		1 => LEFT | TOP,
		2 => LEFT | BOTTOM,
		3 => TOP,
		4 => BOTTOM,
		5 => RIGHT | TOP,
		6 => RIGHT | BOTTOM,
		_ => 0,
	}
}

unsafe extern "C" fn handle_inputs(data: *mut u32) {
	if adm::WINDOW_HANDLE.is_none() {
		return;
	};
	if SDL_STATE.is_none() {
		let deadzone = if let Some(config) = &CONFIG {
			config.deadzone
		} else {
			0.01
		};
		SDL_STATE = poll::PollState::new(adm::WINDOW_HANDLE.unwrap(), deadzone).ok();
	}
	let sdl = SDL_STATE.as_mut().unwrap();
	let keyconfig = KEYCONFIG.as_ref().unwrap();
	sdl.update();

	if sdl.is_tapped(&keyconfig.card_insert) {
		if let Ok(card_data) = std::fs::read("card.bin") {
			card::CARD_DATA.extend(card_data);
		} else {
			println!("Cannot open card.bin");
		}
	}

	data.byte_add(0x24).write(0);

	let mut bits = 0_u32;
	if sdl.is_down(&keyconfig.test) > 0.0 {
		bits |= 0x20000000;
	}
	if sdl.is_down(&keyconfig.service) > 0.0 {
		bits |= 0x80000000;
	}
	if sdl.is_down(&keyconfig.quit) > 0.0 {
		bits |= 0x20000;
	}
	if sdl.is_down(&keyconfig.perspective) > 0.0 {
		bits |= 0x40;
	}
	if sdl.is_down(&keyconfig.intrude) > 0.0 {
		bits |= 0x80;
	}
	if sdl.is_down(&keyconfig.brake) > 0.0 {
		bits |= 0x10;
	}
	if sdl.is_down(&keyconfig.gas) > 0.0 {
		bits |= 0x20;
	}
	if sdl.is_tapped(&keyconfig.gear_next) {
		if GEAR_INDEX < 6 {
			GEAR_INDEX += 1;
		}
	} else if sdl.is_tapped(&keyconfig.gear_previous) {
		if GEAR_INDEX > 0 {
			GEAR_INDEX -= 1;
		}
	} else if sdl.is_tapped(&keyconfig.gear_neutral) {
		GEAR_INDEX = 0;
	} else if sdl.is_tapped(&keyconfig.gear_first) {
		GEAR_INDEX = 1;
	} else if sdl.is_tapped(&keyconfig.gear_second) {
		GEAR_INDEX = 2;
	} else if sdl.is_tapped(&keyconfig.gear_third) {
		GEAR_INDEX = 3;
	} else if sdl.is_tapped(&keyconfig.gear_fourth) {
		GEAR_INDEX = 4;
	} else if sdl.is_tapped(&keyconfig.gear_fifth) {
		GEAR_INDEX = 5;
	} else if sdl.is_tapped(&keyconfig.gear_sixth) {
		GEAR_INDEX = 6;
	}
	bits |= set_gear_bits(GEAR_INDEX);
	data.byte_add(0x8).write(bits);

	let n2jvio = hook::get_symbol("n2jvio") as *mut u16;
	let wheel_left = sdl.is_down(&keyconfig.wheel_left);
	let wheel_right = sdl.is_down(&keyconfig.wheel_right);
	n2jvio.byte_add(0x1A8).write(
		(i16::MAX as f32 - (wheel_left * i16::MAX as f32) + (wheel_right * i16::MAX as f32)) as u16,
	);
	data.byte_add(0x20).write(u32::from_le_bytes(
		(0.0 - wheel_left + wheel_right).to_le_bytes(),
	));

	let gas = sdl.is_down(&keyconfig.gas);
	n2jvio.byte_add(0x1AA).write((gas * i16::MAX as f32) as u16);
	data.byte_add(0x30)
		.write(u32::from_le_bytes(gas.to_le_bytes()));

	let brake = sdl.is_down(&keyconfig.brake);
	n2jvio
		.byte_add(0x1AC)
		.write((brake * i16::MAX as f32) as u16);
	data.byte_add(0x34)
		.write(u32::from_le_bytes(brake.to_le_bytes()));
}

pub unsafe fn init() {
	hook::hook_symbol("_ZN10clSystemN24initEb", adachi as *const ());
	hook::hook_symbol("_ZN10clSystemN212initSystemN2Ev", adachi as *const ());
	hook::hook_symbol("_ZN18clInputDeviceJamma8checkUseEv", adachi as *const ());
	hook::hook_symbol(
		"_ZN18clInputDeviceJamma12handleEventsEv",
		handle_inputs as *const (),
	);
	hook::hook_symbol("_ZN16clInputDevicePad12handleEventsEv", adachi as *const ());
	hook::hook_symbol(
		"_ZN16clInputDevicePad13joyButtonDownEPN3Gap7Display12igControllerENS2_7BUTTONSE",
		adachi as *const (),
	);
	hook::hook_symbol(
		"_ZN16clInputDevicePad13joyButtonDownEPN3Gap7Display12igControllerENS2_7BUTTONSE",
		adachi as *const (),
	);
	hook::hook_symbol(
		"_ZN16clInputDevicePad11joyButtonUpEPN3Gap7Display12igControllerENS2_7BUTTONSE",
		adachi as *const (),
	);
	hook::hook_symbol(
		"_ZN16clInputDevicePad8joyStickEPN3Gap7Display12igControllerEtff",
		adachi as *const (),
	);
}
