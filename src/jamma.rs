use device_query::{DeviceQuery, Keycode};

use crate::*;

unsafe extern "C" fn handle_inputs(data: *mut u32) {
	data.byte_add(0x24).write(0);
	// 0x20: Wheel
	// 0x30: Throttle
	// 0x34: Brake

	let mut bits = 0_u32;
	let keys = device_query::DeviceState::new().get_keys();
	if keys.contains(&Keycode::F1) {
		bits |= 0x20000000;
	}
	if keys.contains(&Keycode::F2) {
		bits |= 0x80000000;
	}
	if keys.contains(&Keycode::Escape) {
		bits |= 0x20000;
	}
	if keys.contains(&Keycode::P) {
		bits |= 0x40;
	}
	if keys.contains(&Keycode::I) {
		bits |= 0x80;
	}
	if keys.contains(&Keycode::Key1) {
		bits |= 0x01;
	}
	if keys.contains(&Keycode::Key2) {
		bits |= 0x02;
	}
	if keys.contains(&Keycode::Key3) {
		bits |= 0x04;
	}
	if keys.contains(&Keycode::Key4) {
		bits |= 0x08;
	}
	if keys.contains(&Keycode::S) {
		bits |= 0x10;
	}
	if keys.contains(&Keycode::W) {
		bits |= 0x20;
	}
	data.byte_add(0x8).write(bits);
	/*
	0x8 bitset:
		0x20000000 = TEST
		0x80000000 = SERVICE
		0x20000 = QUIT
		0x01 = GearIsLeftColumn
		0x02 = GearIsRightColumn
		0x04 = GearIsTopRow
		0x08 = GearIsBottomRow
		0x10 = Brake (Button)
		0x20 = Gas (Button)
		0x40 = Perspective
		0x80 = Intrude
	*/

	let n2jvio = hook::get_symbol("n2jvio") as *mut u16;
	if keys.contains(&Keycode::A) {
		n2jvio.byte_add(0x1A8).write(u16::MIN);
		data.byte_add(0x20)
			.write(u32::from_le_bytes((-1f32).to_le_bytes()));
	} else if keys.contains(&Keycode::D) {
		n2jvio.byte_add(0x1A8).write(u16::MAX);
		data.byte_add(0x20)
			.write(u32::from_le_bytes(1f32.to_le_bytes()));
	} else {
		n2jvio.byte_add(0x1A8).write(u16::MAX / 2);
		data.byte_add(0x20)
			.write(u32::from_le_bytes(0f32.to_le_bytes()));
	}
	if keys.contains(&Keycode::W) {
		n2jvio.byte_add(0x1AA).write(u16::MAX);
		data.byte_add(0x30)
			.write(u32::from_le_bytes(1f32.to_le_bytes()));
	} else {
		n2jvio.byte_add(0x1AA).write(u16::MIN);
		data.byte_add(0x30)
			.write(u32::from_le_bytes(0f32.to_le_bytes()));
	}
	if keys.contains(&Keycode::S) {
		n2jvio.byte_add(0x1AC).write(u16::MAX);
		data.byte_add(0x34)
			.write(u32::from_le_bytes(1f32.to_le_bytes()));
	} else {
		n2jvio.byte_add(0x1AC).write(u16::MIN);
		data.byte_add(0x34)
			.write(u32::from_le_bytes(0f32.to_le_bytes()));
	}
	// 0x1A8: Wheel
	// 0x1AA: Throttle
	// 0x1AC: Brake
}

pub unsafe fn init() {
	hook::hook_symbol("_ZN10clSystemN24initEb", adachi as *const ());
	hook::hook_symbol("_ZN10clSystemN212initSystemN2Ev", adachi as *const ());
	hook::hook_symbol("_ZN18clInputDeviceJamma8checkUseEv", adachi as *const ());
	hook::hook_symbol(
		"_ZN18clInputDeviceJamma12handleEventsEv",
		handle_inputs as *const (),
	);
}
