use device_query::DeviceQuery;
use device_query::Keycode;
use phf::*;
use sdl2::controller::Button;
use sdl2::event::Event;
use sdl2::*;
use std::collections::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Axis {
	LeftStickLeft,
	LeftStickUp,
	LeftStickDown,
	LeftStickRight,
	RightStickLeft,
	RightStickUp,
	RightStickDown,
	RightStickRight,
	LeftTriggerDown,
	LeftTriggerUp,
	RightTriggerDown,
	RightTriggerUp,
}

#[allow(dead_code)]
pub struct PollState {
	sdl: Sdl,
	video: VideoSubsystem,
	gamepad: GameControllerSubsystem,
	joystick: JoystickSubsystem,
	events: EventPump,
	controllers: BTreeMap<u32, controller::GameController>,
	window: *mut sdl2::sys::SDL_Window,
	deadzone: f32,
	keyboard_state: Vec<Keycode>,
	last_keyboard_state: Vec<Keycode>,
	button_state: Vec<Button>,
	last_button_state: Vec<Button>,
	axis_state: BTreeMap<Axis, f32>,
	last_axis_state: BTreeMap<Axis, f32>,
}

#[derive(Clone)]
pub enum KeyBinding {
	Keycode(Keycode),
	Button(Button),
	Axis(Axis),
}

pub struct KeyBindings {
	keys: Vec<KeyBinding>,
}

const MAPPINGS: Map<&'static str, KeyBinding> = phf_map! {
	"F1" => KeyBinding::Keycode(Keycode::F1),
	"F2" => KeyBinding::Keycode(Keycode::F2),
	"F3" => KeyBinding::Keycode(Keycode::F3),
	"F4" => KeyBinding::Keycode(Keycode::F4),
	"F5" => KeyBinding::Keycode(Keycode::F5),
	"F6" => KeyBinding::Keycode(Keycode::F6),
	"F7" => KeyBinding::Keycode(Keycode::F7),
	"F8" => KeyBinding::Keycode(Keycode::F8),
	"F9" => KeyBinding::Keycode(Keycode::F9),
	"F10" => KeyBinding::Keycode(Keycode::F10),
	"F11" => KeyBinding::Keycode(Keycode::F11),
	"F12" => KeyBinding::Keycode(Keycode::F12),
	"NUM0" => KeyBinding::Keycode(Keycode::Key0),
	"NUM1" => KeyBinding::Keycode(Keycode::Key1),
	"NUM2" => KeyBinding::Keycode(Keycode::Key2),
	"NUM3" => KeyBinding::Keycode(Keycode::Key3),
	"NUM4" => KeyBinding::Keycode(Keycode::Key4),
	"NUM5" => KeyBinding::Keycode(Keycode::Key5),
	"NUM6" => KeyBinding::Keycode(Keycode::Key6),
	"NUM7" => KeyBinding::Keycode(Keycode::Key7),
	"NUM8" => KeyBinding::Keycode(Keycode::Key8),
	"NUM9" => KeyBinding::Keycode(Keycode::Key9),
	"UPARROW" => KeyBinding::Keycode(Keycode::Up),
	"LEFTARROW" => KeyBinding::Keycode(Keycode::Left),
	"DOWNARROW" => KeyBinding::Keycode(Keycode::Down),
	"RIGHTARROW" => KeyBinding::Keycode(Keycode::Right),
	"ENTER" => KeyBinding::Keycode(Keycode::Enter),
	"SPACE" => KeyBinding::Keycode(Keycode::Space),
	"CONTROL" => KeyBinding::Keycode(Keycode::LControl),
	"SHIFT" => KeyBinding::Keycode(Keycode::LShift),
	"TAB" => KeyBinding::Keycode(Keycode::Tab),
	"ESCAPE" => KeyBinding::Keycode(Keycode::Escape),
	"A" => KeyBinding::Keycode(Keycode::A),
	"B" => KeyBinding::Keycode(Keycode::B),
	"C" => KeyBinding::Keycode(Keycode::C),
	"D" => KeyBinding::Keycode(Keycode::D),
	"E" => KeyBinding::Keycode(Keycode::E),
	"F" => KeyBinding::Keycode(Keycode::F),
	"G" => KeyBinding::Keycode(Keycode::G),
	"H" => KeyBinding::Keycode(Keycode::H),
	"I" => KeyBinding::Keycode(Keycode::I),
	"J" => KeyBinding::Keycode(Keycode::J),
	"K" => KeyBinding::Keycode(Keycode::K),
	"L" => KeyBinding::Keycode(Keycode::L),
	"M" => KeyBinding::Keycode(Keycode::M),
	"N" => KeyBinding::Keycode(Keycode::N),
	"O" => KeyBinding::Keycode(Keycode::O),
	"P" => KeyBinding::Keycode(Keycode::P),
	"Q" => KeyBinding::Keycode(Keycode::Q),
	"R" => KeyBinding::Keycode(Keycode::R),
	"S" => KeyBinding::Keycode(Keycode::S),
	"T" => KeyBinding::Keycode(Keycode::T),
	"U" => KeyBinding::Keycode(Keycode::U),
	"V" => KeyBinding::Keycode(Keycode::V),
	"W" => KeyBinding::Keycode(Keycode::W),
	"X" => KeyBinding::Keycode(Keycode::X),
	"Y" => KeyBinding::Keycode(Keycode::Y),
	"Z" => KeyBinding::Keycode(Keycode::Z),
	"SDL_A" => KeyBinding::Button(Button::A),
	"SDL_B" => KeyBinding::Button(Button::B),
	"SDL_X" => KeyBinding::Button(Button::X),
	"SDL_Y" => KeyBinding::Button(Button::Y),
	"SDL_BACK" => KeyBinding::Button(Button::Back),
	"SDL_GUIDE" => KeyBinding::Button(Button::Guide),
	"SDL_START" => KeyBinding::Button(Button::Start),
	"SDL_LSHOULDER" => KeyBinding::Button(Button::LeftShoulder),
	"SDL_RSHOULDER" => KeyBinding::Button(Button::RightShoulder),
	"SDL_DPAD_UP" => KeyBinding::Button(Button::DPadUp),
	"SDL_DPAD_LEFT" => KeyBinding::Button(Button::DPadLeft),
	"SDL_DPAD_DOWN" => KeyBinding::Button(Button::DPadDown),
	"SDL_DPAD_RIGHT" => KeyBinding::Button(Button::DPadRight),
	"SDL_MISC" => KeyBinding::Button(Button::Misc1),
	"SDL_PADDLE1" => KeyBinding::Button(Button::Paddle1),
	"SDL_PADDLE2" => KeyBinding::Button(Button::Paddle2),
	"SDL_PADDLE3" => KeyBinding::Button(Button::Paddle3),
	"SDL_PADDLE4" => KeyBinding::Button(Button::Paddle4),
	"SDL_TOUCHPAD" => KeyBinding::Button(Button::Touchpad),
	"SDL_LSTICK_PRESS" => KeyBinding::Button(Button::LeftStick),
	"SDL_RSTICK_PRESS" => KeyBinding::Button(Button::RightStick),
	"SDL_LSTICK_LEFT" => KeyBinding::Axis(Axis::LeftStickLeft),
	"SDL_LSTICK_UP" => KeyBinding::Axis(Axis::LeftStickUp),
	"SDL_LSTICK_DOWN" => KeyBinding::Axis(Axis::LeftStickDown),
	"SDL_LSTICK_RIGHT" => KeyBinding::Axis(Axis::LeftStickRight),
	"SDL_RSTICK_LEFT" => KeyBinding::Axis(Axis::RightStickLeft),
	"SDL_RSTICK_UP" => KeyBinding::Axis(Axis::RightStickUp),
	"SDL_RSTICK_DOWN" => KeyBinding::Axis(Axis::RightStickDown),
	"SDL_RSTICK_RIGHT" => KeyBinding::Axis(Axis::RightStickRight),
	"SDL_LTRIGGER_DOWN" => KeyBinding::Axis(Axis::LeftTriggerDown),
	"SDL_LTRIGGER_UP" => KeyBinding::Axis(Axis::LeftTriggerUp),
	"SDL_RTRIGGER_DOWN" => KeyBinding::Axis(Axis::RightTriggerDown),
	"SDL_RTRIGGER_UP" => KeyBinding::Axis(Axis::RightTriggerUp),
};

pub fn parse_keybinding(toml: Vec<String>) -> KeyBindings {
	let mut keybindings = KeyBindings { keys: Vec::new() };
	for value in toml {
		if let Some(keybinding) = MAPPINGS.get(&value) {
			keybindings.keys.push(keybinding.clone());
		} else {
			panic!("Incorrect keybinding {value}");
		}
	}
	keybindings
}

impl PollState {
	pub fn new(handle: *const libc::c_void, axis_deadzone: f32) -> Result<Self, String> {
		let sdl = sdl2::init()?;
		let video = sdl.video()?;
		let joystick = sdl.joystick()?;
		let gamepad = sdl.game_controller()?;
		let events = sdl.event_pump()?;

		gamepad
			.load_mappings("gamecontrollerdb.txt")
			.map_err(|_| "Failed to parse gamecontrollerdb.txt")?;
		let mut controllers = BTreeMap::new();
		for i in 0..gamepad.num_joysticks()? {
			let controller = gamepad.open(i).map_err(|_| {
				format!(
					"Failed to open {}",
					gamepad.name_for_index(i).map_or(
						String::from("Failed to get controller information"),
						|name| name
					)
				)
			})?;
			controllers.insert(i, controller);
		}
		let window = unsafe { sdl2::sys::SDL_CreateWindowFrom(handle) };

		Ok(Self {
			sdl,
			video,
			gamepad,
			joystick,
			events,
			controllers,
			window,
			deadzone: axis_deadzone,
			keyboard_state: Vec::with_capacity(255),
			last_keyboard_state: Vec::with_capacity(255),
			button_state: Vec::with_capacity(32),
			last_button_state: Vec::with_capacity(32),
			axis_state: BTreeMap::new(),
			last_axis_state: BTreeMap::new(),
		})
	}

	pub fn update(&mut self) {
		self.last_keyboard_state.clear();
		self.last_button_state.clear();
		self.last_axis_state.clear();

		self.last_keyboard_state.extend(&self.keyboard_state);
		self.last_button_state.extend(&self.button_state);
		self.last_axis_state.extend(&self.axis_state);

		self.keyboard_state = device_query::DeviceState::new().get_keys();

		for event in self.events.poll_iter() {
			match event {
				Event::ControllerDeviceAdded {
					timestamp: _,
					which,
				} => {
					let controller = self.gamepad.open(which).unwrap();
					self.controllers.insert(which, controller);
				}
				Event::ControllerDeviceRemoved {
					timestamp: _,
					which,
				} => {
					if let Some(controller) = self.controllers.remove(&which) {
						drop(controller);
					}
				}
				Event::ControllerButtonDown {
					timestamp: _,
					which: _,
					button,
				} => {
					self.button_state.push(button);
				}
				Event::ControllerButtonUp {
					timestamp: _,
					which: _,
					button,
				} => {
					self.button_state.retain(|b| b != &button);
				}
				Event::ControllerAxisMotion {
					timestamp: _,
					which: _,
					axis,
					value,
				} => {
					let value = value as f32 / i16::MAX as f32;
					use Axis::*;
					if value > self.deadzone {
						let axis = match axis {
							controller::Axis::LeftX => LeftStickRight,
							controller::Axis::LeftY => LeftStickDown,
							controller::Axis::RightX => RightStickRight,
							controller::Axis::RightY => RightStickDown,
							controller::Axis::TriggerLeft => LeftTriggerDown,
							controller::Axis::TriggerRight => RightTriggerDown,
						};
						
						if axis == LeftStickRight {
							self.axis_state.insert(LeftStickLeft, 0.0);
						}

						if axis == RightStickRight {
							self.axis_state.insert(RightStickLeft, 0.0);
						}
						
						self.axis_state.insert(axis, value);
					} else if value < -self.deadzone {
						let axis = match axis {
							controller::Axis::LeftX => LeftStickLeft,
							controller::Axis::LeftY => LeftStickUp,
							controller::Axis::RightX => RightStickLeft,
							controller::Axis::RightY => RightStickUp,
							controller::Axis::TriggerLeft => LeftTriggerUp,
							controller::Axis::TriggerRight => RightTriggerUp,
						};
						
						if axis == LeftStickLeft {
							self.axis_state.insert(LeftStickRight, 0.0);
						}

						if axis == RightStickLeft {
							self.axis_state.insert(RightStickRight, 0.0);
						}

						self.axis_state.insert(axis, -value);
					} else {
						let (axis_positive, axis_negative) = match axis {
							controller::Axis::LeftX => (LeftStickRight, LeftStickLeft),
							controller::Axis::LeftY => (LeftStickDown, LeftStickUp),
							controller::Axis::RightX => (RightStickRight, RightStickLeft),
							controller::Axis::RightY => (RightStickDown, RightStickUp),
							controller::Axis::TriggerLeft => (LeftTriggerDown, RightTriggerDown),
							controller::Axis::TriggerRight => (RightTriggerDown, RightTriggerUp),
						};
						self.axis_state.insert(axis_positive, 0.0);
						self.axis_state.insert(axis_negative, 0.0);
					}
				}
				Event::KeyDown {
					timestamp: _,
					window_id: _,
					keycode: _,
					scancode: _,
					keymod: _,
					repeat: _,
				} => {
					// Currently this is broken and waiting on sdl 3.20, see #5142 for updates
					// for now im using device_query
				}
				_ => {}
			}
		}
	}

	fn keycode_is_down(&self, keycode: &Keycode) -> bool {
		self.keyboard_state.contains(keycode)
	}
	fn keycode_is_up(&self, keycode: &Keycode) -> bool {
		!self.keyboard_state.contains(keycode)
	}
	fn keycode_was_down(&self, keycode: &Keycode) -> bool {
		self.last_keyboard_state.contains(keycode)
	}
	fn keycode_was_up(&self, keycode: &Keycode) -> bool {
		!self.last_keyboard_state.contains(keycode)
	}
	fn keycode_is_tapped(&self, keycode: &Keycode) -> bool {
		self.keycode_is_down(keycode) && self.keycode_was_up(keycode)
	}
	fn keycode_is_released(&self, keycode: &Keycode) -> bool {
		self.keycode_is_up(keycode) && self.keycode_was_down(keycode)
	}

	fn button_is_down(&self, button: &Button) -> bool {
		self.button_state.contains(button)
	}
	fn button_is_up(&self, button: &Button) -> bool {
		!self.button_state.contains(&button)
	}
	fn button_was_down(&self, button: &Button) -> bool {
		self.last_button_state.contains(&button)
	}
	fn button_was_up(&self, button: &Button) -> bool {
		!self.last_button_state.contains(&button)
	}
	fn button_is_tapped(&self, button: &Button) -> bool {
		self.button_is_down(button) && self.button_was_up(button)
	}
	fn button_is_released(&self, button: &Button) -> bool {
		self.button_was_down(button) && self.button_is_up(button)
	}

	fn axis_is_down(&self, axis: &Axis) -> f32 {
		*self.axis_state.get(axis).unwrap_or(&0.0)
	}
	fn axis_is_up(&self, axis: &Axis) -> bool {
		self.axis_is_down(axis) == 0.0
	}
	fn axis_was_down(&self, axis: &Axis) -> f32 {
		*self.last_axis_state.get(axis).unwrap_or(&0.0)
	}
	fn axis_was_up(&self, axis: &Axis) -> bool {
		self.axis_was_down(axis) == 0.0
	}
	fn axis_is_tapped(&self, axis: &Axis) -> bool {
		self.axis_is_down(axis) != 0.0 && self.axis_was_up(axis)
	}
	fn axis_is_released(&self, axis: &Axis) -> bool {
		self.axis_was_down(axis) != 0.0 && self.axis_is_up(axis)
	}

	fn binding_is_down(&self, keybinding: &KeyBinding) -> f32 {
		match keybinding {
			KeyBinding::Keycode(keycode) => self.keycode_is_down(keycode) as i32 as f32,
			KeyBinding::Button(button) => self.button_is_down(button) as i32 as f32,
			KeyBinding::Axis(axis) => self.axis_is_down(axis),
		}
	}
	fn binding_is_up(&self, keybinding: &KeyBinding) -> bool {
		match keybinding {
			KeyBinding::Keycode(keycode) => self.keycode_is_up(keycode),
			KeyBinding::Button(button) => self.button_is_up(button),
			KeyBinding::Axis(axis) => self.axis_is_up(axis),
		}
	}
	fn binding_was_down(&self, keybinding: &KeyBinding) -> f32 {
		match keybinding {
			KeyBinding::Keycode(keycode) => self.keycode_was_down(keycode) as i32 as f32,
			KeyBinding::Button(button) => self.button_was_down(button) as i32 as f32,
			KeyBinding::Axis(axis) => self.axis_was_down(axis),
		}
	}
	fn binding_was_up(&self, keybinding: &KeyBinding) -> bool {
		match keybinding {
			KeyBinding::Keycode(keycode) => self.keycode_was_up(keycode),
			KeyBinding::Button(button) => self.button_was_up(button),
			KeyBinding::Axis(axis) => self.axis_was_up(axis),
		}
	}
	fn binding_is_tapped(&self, keybinding: &KeyBinding) -> bool {
		match keybinding {
			KeyBinding::Keycode(keycode) => self.keycode_is_tapped(keycode),
			KeyBinding::Button(button) => self.button_is_tapped(button),
			KeyBinding::Axis(axis) => self.axis_is_tapped(axis),
		}
	}
	fn binding_is_released(&self, keybinding: &KeyBinding) -> bool {
		match keybinding {
			KeyBinding::Keycode(keycode) => self.keycode_is_released(keycode),
			KeyBinding::Button(button) => self.button_is_released(button),
			KeyBinding::Axis(axis) => self.axis_is_released(axis),
		}
	}

	pub fn is_down(&self, keybindings: &KeyBindings) -> f32 {
		for keybinding in keybindings.keys.iter() {
			let value = self.binding_is_down(keybinding);
			if value > 0.0 {
				return value;
			}
		}
		0.0
	}
	pub fn is_up(&self, keybindings: &KeyBindings) -> bool {
		for keybinding in keybindings.keys.iter() {
			if self.binding_is_up(keybinding) {
				return true;
			}
		}
		false
	}
	pub fn was_down(&self, keybindings: &KeyBindings) -> f32 {
		for keybinding in keybindings.keys.iter() {
			let value = self.binding_was_down(keybinding);
			if value > 0.0 {
				return value;
			}
		}
		0.0
	}
	pub fn was_up(&self, keybindings: &KeyBindings) -> bool {
		for keybinding in keybindings.keys.iter() {
			if self.binding_was_up(keybinding) {
				return true;
			}
		}
		false
	}
	pub fn is_tapped(&self, keybindings: &KeyBindings) -> bool {
		for keybinding in keybindings.keys.iter() {
			if self.binding_is_tapped(keybinding) {
				return true;
			}
		}
		false
	}
	pub fn is_released(&self, keybindings: &KeyBindings) -> bool {
		for keybinding in keybindings.keys.iter() {
			if self.binding_is_released(keybinding) {
				return true;
			}
		}
		false
	}
}
