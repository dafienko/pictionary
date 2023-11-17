use std::net::{TcpStream, TcpListener};
use std::env;

use crate::message::{GameMessage, parse_game_message};
use piston_window::*;

/*

game = connect_players()

init window

message pump {
	if let action {
		game.process_action(action)
	}

	game.render()
}

*/

pub enum GameAction {
	Tick(),
	TypeLetter(u8),
	DeleteLetter(),
	Draw(u32, u32),
	Enter(),
}


trait Player {
	fn render(self: &Self);
	fn process_action(self: &Self, action: GameAction) -> Option<Box<dyn Player>>;
}

enum Drawer {
	Init,
	PickingWord(Vec<String>),
	Drawing(String, u32)
}

impl Player for Drawer {
	fn render(self: &Self) {

	}

	fn process_action(self: &Self, action: GameAction) -> Option<Box<dyn Player>> {

		None
	}
}

enum Guesser {
	Init,
	WaitingForDrawer,
	Guessing(String, String)
}

impl Player for Guesser {
	fn render(self: &Self) {

	}

	fn process_action(self: &Self, action: GameAction) -> Option<Box<dyn Player>> {

		None
	}
}

pub struct Game {
	stream: TcpStream,
	role: Box<dyn Player>,

	mouse_down: bool,
}

impl Game {
	pub fn new() -> Self {
		let args: Vec<String> = env::args().collect();
		let res = if args.len() >= 2 {
			let ip = args.get(1).unwrap();
			println!("connecting to {}...", ip);
			(TcpStream::connect(ip).unwrap(), Box::new(Guesser::Init) as Box<dyn Player>)
		} else {
			println!("waiting for connection...");
			(TcpListener::bind("127.0.0.1:4912").unwrap().accept().unwrap().0, Box::new(Drawer::Init) as Box<dyn Player>)
		};

		Game {
			stream: res.0,
			role: res.1,
			mouse_down: false,
		}
	}

	fn process_message(&self, message: GameMessage) {
		match message {
			GameMessage::PutPixel(x, y) => {
				
			}
		}
	}

	pub fn start_message_listener(&self) {
		let mut reader = self.stream.try_clone().unwrap();
		loop {
			let message = parse_game_message(&mut reader);
			self.process_message(message);
		}
	}

	pub fn process_action(&mut self, action: GameAction) {
		if let Some(new) = self.role.process_action(action) {
			self.role = new;
		};
	}

	pub fn process_event(&mut self, e: Event) {
		if self.mouse_down {
			if let Some(p) = e.mouse_cursor_args() {
				let x = p[0] as u32;
				let y = p[1] as u32;
				self.process_action(GameAction::Draw(x, y));
			}
		}

		if let Event::Input(input, _) = e.clone() {
			if let Input::Button(args) = input {
				if let Button::Mouse(mouse_button) = args.button {
					if let MouseButton::Left = mouse_button {
						self.mouse_down = match args.state {
							ButtonState::Press => true,
							ButtonState::Release => false,
						}
					}
				}
			}
		}
	}

	pub fn render(&self, c: Context, g: &mut G2d, d: &mut gfx_device_gl::Device) {
		self.role.render();
		/*
		texture_context.encoder.flush(device);
				glyphs.factory.encoder.flush(device);

                clear([1.0; 4], g);

                image(&texture, c.transform.scale(4.0, 4.0), g);

				let mut center_text = |render_text: &str, x: f64, y: f64| {
					let w = metrics(&font, render_text, &mut glyphs);
					font.draw(
						render_text,
						&mut glyphs,
						&c.draw_state,
						c.transform.trans(x - w * 0.5, y), g
					).unwrap();
				};
		 */
	}
}

