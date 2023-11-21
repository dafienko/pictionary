extern crate piston_window;
extern crate image as im;

use std::net::TcpStream;

use crate::game::game_action::GameAction;
use crate::canvas::CanvasOperation;
use crate::game::message::GameMessage;
use piston_window::*;
use std::sync::mpsc::Sender;

enum KeyboardButtonType {
	Letter(char),
	Number(u8),
	Enter,
	Backspace,
	Other
}

impl KeyboardButtonType {
	fn from_key(key: Key) -> KeyboardButtonType {
		match key {
			Key::A => KeyboardButtonType::Letter('a'),
			Key::B => KeyboardButtonType::Letter('b'),
			Key::C => KeyboardButtonType::Letter('c'),
			Key::D => KeyboardButtonType::Letter('d'),
			Key::E => KeyboardButtonType::Letter('e'),
			Key::F => KeyboardButtonType::Letter('f'),
			Key::G => KeyboardButtonType::Letter('g'),
			Key::H => KeyboardButtonType::Letter('h'),
			Key::I => KeyboardButtonType::Letter('i'),
			Key::J => KeyboardButtonType::Letter('j'),
			Key::K => KeyboardButtonType::Letter('k'),
			Key::L => KeyboardButtonType::Letter('l'),
			Key::M => KeyboardButtonType::Letter('m'),
			Key::N => KeyboardButtonType::Letter('n'),
			Key::O => KeyboardButtonType::Letter('o'),
			Key::P => KeyboardButtonType::Letter('p'),
			Key::Q => KeyboardButtonType::Letter('q'),
			Key::R => KeyboardButtonType::Letter('r'),
			Key::S => KeyboardButtonType::Letter('s'),
			Key::T => KeyboardButtonType::Letter('t'),
			Key::U => KeyboardButtonType::Letter('u'),
			Key::V => KeyboardButtonType::Letter('v'),
			Key::W => KeyboardButtonType::Letter('w'),
			Key::X => KeyboardButtonType::Letter('x'),
			Key::Y => KeyboardButtonType::Letter('y'),
			Key::Z => KeyboardButtonType::Letter('z'),

			Key::D0 => KeyboardButtonType::Number(0),
			Key::D1 => KeyboardButtonType::Number(1),
			Key::D2 => KeyboardButtonType::Number(2),
			Key::D3 => KeyboardButtonType::Number(3),
			Key::D4 => KeyboardButtonType::Number(4),
			Key::D5 => KeyboardButtonType::Number(5),
			Key::D6 => KeyboardButtonType::Number(6),
			Key::D7 => KeyboardButtonType::Number(7),
			Key::D8 => KeyboardButtonType::Number(8),
			Key::D9 => KeyboardButtonType::Number(9),

			Key::Return => KeyboardButtonType::Enter,

			Key::Backspace => KeyboardButtonType::Backspace,

			_ => KeyboardButtonType::Other,
		}
	}
}


struct EventState {
	last_mouse_pos: (u32, u32),
	current_mouse_pos: (u32, u32),
	left_mouse_down: bool,
	right_mouse_down: bool
}

pub struct Communications {
	stream: Option<TcpStream>,
	action_sender: Sender<GameAction>,
	canvas_op_sender: Sender<CanvasOperation>,
	event_state: EventState,
}

impl Communications {
	pub fn new(stream: Option<TcpStream>, action_sender: Sender<GameAction>, canvas_op_sender: Sender<CanvasOperation>) -> Self {
		Communications { 
			stream: stream, 
			action_sender: action_sender, 
			canvas_op_sender: canvas_op_sender, 
			event_state: EventState {
				last_mouse_pos: (0, 0),
				current_mouse_pos: (0, 0),
				left_mouse_down: false, 
				right_mouse_down: false 
			} 
		}
	}

	pub fn set_stream(&mut self, stream: TcpStream) {
		self.stream = Some(stream);
	}

	pub fn send_action(&mut self, action: GameAction) {
		self.action_sender.send(action).unwrap();
	}

	pub fn send_canvas_op(&mut self, op: CanvasOperation) {
		self.canvas_op_sender.send(op).unwrap();
	}

	pub fn send_message(&mut self, message: GameMessage) {
		if let Some(mut stream) = self.stream.as_mut() {
			message.send(&mut stream);
		}
	}

	fn process_keyboard_button_event(&mut self, keyboard_button_type: KeyboardButtonType) {
		match keyboard_button_type {
			KeyboardButtonType::Letter(char) => {
				self.send_action(GameAction::TypeLetter(char));
			},

			KeyboardButtonType::Number(num) => {
				self.send_action(GameAction::TypeNumber(num));
			},

			KeyboardButtonType::Enter => {
				self.send_action(GameAction::Enter);
			},

			KeyboardButtonType::Backspace => {
				self.send_action(GameAction::DeleteLetter);
			},

			_ => {}
		};
	}

	fn process_button_event(&mut self, args: ButtonArgs) {
		match args.button {
			Button::Keyboard(key) => {
				if let ButtonState::Press = args.state {
					self.process_keyboard_button_event(KeyboardButtonType::from_key(key));
				}
			},

			Button::Mouse(mouse_button) => {
				let state = match args.state {
					ButtonState::Press => true,
					ButtonState::Release => false,
				};
				
				match mouse_button {
					MouseButton::Left => {
						self.event_state.left_mouse_down = state;
						if state {
							self.send_action(GameAction::LeftClick(self.event_state.current_mouse_pos.0, self.event_state.current_mouse_pos.1))
						}
					},
					
					MouseButton::Right => {
						self.event_state.right_mouse_down = state;
						if state {
							self.send_action(GameAction::RightClick(self.event_state.current_mouse_pos.0, self.event_state.current_mouse_pos.1))
						}
					},
					
					_ => {}
				};
			},
			
			_ => {}
		}
	}

	pub fn process_event(&mut self, e: Event) {
		if let Some(p) = e.mouse_cursor_args() {
			let l = self.event_state.current_mouse_pos;
			let c = ((p[0] as u32) / 8, (p[1] as u32) / 8);

			self.event_state.last_mouse_pos = l;
			self.event_state.current_mouse_pos = c;

			if self.event_state.left_mouse_down {
				self.send_action(GameAction::LeftClickDrag(l.0, l.1, c.0, c.1));
			}

			if self.event_state.right_mouse_down {
				self.send_action(GameAction::RightClickDrag(l.0, l.1, c.0, c.1));
			}
		}

		match e {
			Event::Input(input, _) => {
				if let Input::Button(args) = input {
					self.process_button_event(args);
				};
			},

			Event::Loop(loop_event) => {
				if let Loop::Update(update_args) = loop_event {
					self.action_sender.send(GameAction::Update(update_args.dt)).unwrap();
				}
			}
			_ => {},
		};
	}
}