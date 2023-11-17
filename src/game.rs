extern crate piston_window;
extern crate image as im;
#[path = "text_util.rs"] mod text_util;

use std::net::{TcpStream, TcpListener};
use std::env;

use text_util::metrics;
use gfx_device_gl::{Factory, Resources, CommandBuffer};
use crate::canvas::CanvasOperation;
use crate::message::{GameMessage, parse_game_message};
use piston_window::*;
use piston_window::glyph_cache::rusttype::GlyphCache;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;

#[derive(Debug)]
pub enum GameAction {
	Tick(),
	TypeLetter(u8),
	DeleteLetter(),
	Draw(u32, u32),
	Click(u32, u32),
	Enter(),
}

type Glyphs<'a> = GlyphCache<'a, TextureContext<Factory, Resources, CommandBuffer>, Texture<Resources>>;

fn center_text(font: &mut Text, glyphs: &mut Glyphs<'_>, text: &str, x: f64, y: f64, c: Context, g: &mut G2d) {
	let w = metrics(font, text, glyphs);
	font.draw(
		text,
		glyphs,
		&c.draw_state,
		c.transform.trans(x - w * 0.5, y), g
	).unwrap();
}

trait Player {
	fn render(self: &Self, game: &Game, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device);

	fn process_action(self: &Self, communications: &mut Communications, action: GameAction) -> Option<Box<dyn Player + Send>>;
}

enum Drawer {
	Init,
	PickingWord(Vec<String>),
	Drawing(String, u32)
}

impl Player for Drawer {
	fn render(self: &Self, game: &Game, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device) {
		center_text(font, glyphs, "Drawer", 200.0, 50.0, c, g);
	}

	fn process_action(self: &Self, communications: &mut Communications, action: GameAction) -> Option<Box<dyn Player + Send>> {
		match action {
			GameAction::Click(x, y) => {
				GameMessage::PutPixel(x, y).send(&mut communications.stream);
				communications.action_sender.send(GameAction::Draw(x, y)).unwrap();
				None
			}
			_ => None
		}
	}
}

enum Guesser {
	Init,
	WaitingForDrawer,
	Guessing(String, String)
}

impl Player for Guesser {
	fn render(self: &Self, game: &Game, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device) {
		center_text(font, glyphs, "Guesser", 200.0, 50.0, c, g);
	}

	fn process_action(self: &Self, communications: &mut Communications, action: GameAction) -> Option<Box<dyn Player + Send>> {
		match action {
			_ => None
		}
	}
}

struct WaitingPlayer {}

impl Player for WaitingPlayer {
	fn render(self: &Self, _game: &Game, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, _device: &mut gfx_device_gl::Device) {
		center_text(font, glyphs, "Connecting...", 200.0, 50.0, c, g);
	}

	fn process_action(self: &Self, _communications: &mut Communications, _action: GameAction) -> Option<Box<dyn Player + Send>> { None }
}

struct EventState {
	mouse_down: bool,
}

struct Communications {
	stream: TcpStream,
	action_sender: Sender<GameAction>,
	canvas_op_sender: Sender<CanvasOperation>,
}

pub struct Game {
	communications: Option<Communications>,
	role: Box<dyn Player + Send>,
	event_state: EventState,
}

impl Game {
	pub fn new(canvas_op_sender: Sender<CanvasOperation>) -> Arc<Mutex<Self>> {
		let (sender, receiver) = channel();
		let this = Arc::new(Mutex::new(Game {
			communications: None,
			role: Box::new(WaitingPlayer {}),
			event_state: EventState {
				mouse_down: false
			}
		}));

		let connection_thread_ref = this.clone();
		thread::spawn(move || {
			let args: Vec<String> = env::args().collect();
			let (stream, role) = if args.len() >= 2 {
				let ip = args.get(1).unwrap();
				println!("connecting to {}...", ip);
				(TcpStream::connect(ip).unwrap(), Box::new(Guesser::Init) as Box<dyn Player + Send>)
			} else {
				println!("waiting for connection...");
				(TcpListener::bind("127.0.0.1:4912").unwrap().accept().unwrap().0, Box::new(Drawer::Init) as Box<dyn Player + Send>)
			};
			
			let action_sender = sender.clone();
			let mut reader = stream.try_clone().unwrap();
			connection_thread_ref.lock().unwrap().communications = Some(Communications {
				stream: stream, 
				action_sender: sender, 
				canvas_op_sender
			});

			connection_thread_ref.lock().unwrap().role = role;

			thread::spawn(move || {
				loop {
					let message = parse_game_message(&mut reader);
					action_sender.send(Game::translate_message(message)).unwrap();
				}
			});
		});
		
		let action_thread_ref = this.clone();
		thread::spawn(move || {
			loop {
				let action = receiver.recv().unwrap();
				println!("{:?}", action);
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

	pub fn process_action(&mut self, action: GameAction) {
		if let Some(communications) = &mut self.communications {
			match action {
				GameAction::Draw(x, y) => {
					communications.canvas_op_sender.send(CanvasOperation::Pixel(x, y, 0, 0, 0)).unwrap();
				}
				_ => {}
			};
			
			if let Some(new) = self.role.process_action(communications, action) {
				self.role = new;
			}
		}
	}

	pub fn process_event(&mut self, e: Event) {
		if let Some(communications) = &mut self.communications {
			if self.event_state.mouse_down {
				if let Some(p) = e.mouse_cursor_args() {
					let x = p[0] as u32;
					let y = p[1] as u32;

					communications.action_sender.send(GameAction::Click(x / 4, y / 4)).unwrap();
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
	}

	pub fn render(&self, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device) {
		self.role.render(self, font, glyphs, c, g, device);
	}
}

