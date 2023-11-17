use std::net::{TcpStream, TcpListener};
use std::env;

use crate::canvas::GameCanvas;
use crate::message::{GameMessage, parse_game_message};
use piston_window::*;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;

pub enum GameAction {
	Tick(),
	TypeLetter(u8),
	DeleteLetter(),
	Draw(u32, u32),
	Enter(),
}

trait Player {
	fn render(self: &Self, canvas: &mut GameCanvas, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device);
	fn process_action(self: &Self, action: GameAction) -> Option<Box<dyn Player + Send>>;
}

enum Drawer {
	Init,
	PickingWord(Vec<String>),
	Drawing(String, u32)
}

impl Player for Drawer {
	fn render(self: &Self, canvas: &mut GameCanvas, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device) {
		canvas.center_text("Drawer", 200.0, 50.0, c, g);
	}

	fn process_action(self: &Self, action: GameAction) -> Option<Box<dyn Player + Send>> {

		None
	}
}

enum Guesser {
	Init,
	WaitingForDrawer,
	Guessing(String, String)
}

impl Player for Guesser {
	fn render(self: &Self, canvas: &mut GameCanvas, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device) {
		canvas.center_text("Guesser", 200.0, 50.0, c, g);
	}

	fn process_action(self: &Self, action: GameAction) -> Option<Box<dyn Player + Send>> {

		None
	}
}

struct EventState {
	mouse_down: bool,
}

pub struct Game {
	stream: TcpStream,
	action_sender: Sender<GameAction>,
	role: Box<dyn Player + Send>,
	event_state: EventState,
}

impl Game {
	pub fn new() -> Arc<Mutex<Self>> {
		let args: Vec<String> = env::args().collect();
		let (stream, role) = if args.len() >= 2 {
			let ip = args.get(1).unwrap();
			println!("connecting to {}...", ip);
			(TcpStream::connect(ip).unwrap(), Box::new(Guesser::Init) as Box<dyn Player + Send>)
		} else {
			println!("waiting for connection...");
			(TcpListener::bind("127.0.0.1:4912").unwrap().accept().unwrap().0, Box::new(Drawer::Init) as Box<dyn Player + Send>)
		};

		let mut reader = stream.try_clone().unwrap();
		let (sender, receiver) = channel();

		let this = Arc::new(Mutex::new(Game {
			stream,
			action_sender: sender,
			role,
			event_state: EventState {
				mouse_down: false
			}
		}));

		let tcp_thread_ref = this.clone();
		thread::spawn(move || {
			loop {
				let message = parse_game_message(&mut reader);
				tcp_thread_ref.lock().unwrap().push(Game::translate_message(message));
			}
		});
		
		let action_thread_ref = this.clone();
		thread::spawn(move || {
			loop {
				let action = receiver.recv().unwrap();
				action_thread_ref.lock().unwrap().process_action(action);
			}
		});	

		this
	}

	fn translate_message(message: GameMessage) -> GameAction {
		match message {
			GameMessage::PutPixel(x, y) => GameAction::Draw(x, y)
		}
	}

	fn push(&self, action: GameAction) {
		self.action_sender.send(action).unwrap();
	}

	pub fn process_action(&mut self, action: GameAction) {
		if let Some(new) = self.role.process_action(action) {
			self.role = new;
		};
	}

	pub fn process_event(&mut self, e: Event) {
		if self.event_state.mouse_down {
			if let Some(p) = e.mouse_cursor_args() {
				let x = p[0] as u32;
				let y = p[1] as u32;

				self.push(GameAction::Draw(x, y));
			}
		}

		if let Event::Input(input, _) = e.clone() {
			if let Input::Button(args) = input {
				if let Button::Mouse(mouse_button) = args.button {
					if let MouseButton::Left = mouse_button {
						self.event_state.mouse_down = match args.state {
							ButtonState::Press => true,
							ButtonState::Release => false,
						}
					}
				}
			}
		}
	}

	pub fn render(&self, canvas: &mut GameCanvas, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device) {
		self.role.render(canvas, c, g, device);
		
		
		
		/*

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

