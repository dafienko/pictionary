extern crate piston_window;
extern crate image as im;

use std::net::{TcpStream, TcpListener};
use std::env;
use std::cmp;

use rand::seq::SliceRandom;
use gfx_device_gl::{Factory, Resources, CommandBuffer};
use crate::canvas::CanvasOperation;
use crate::message::{GameMessage, parse_game_message};
use piston_window::*;
use piston_window::glyph_cache::rusttype::GlyphCache;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;

const DRAWING_TIME: u32 = 30;

#[derive(Debug)]
pub enum GameAction {
	Update(f64),
	Tick,
	TypeNumber(u8),
	TypeLetter(char),
	DeleteLetter,
	Enter,
	SetTimeRemaining(u32),
	SetWordSkeleton(String),
	Draw(u32, u32),
	Erase(u32, u32),
	LeftClick(u32, u32),
	RightClick(u32, u32),
	Guess(String),
	GuessResult(Option<String>),
	GameOver(String),
	SwapRoles,
}

type Glyphs<'a> = GlyphCache<'a, TextureContext<Factory, Resources, CommandBuffer>, Texture<Resources>>;

pub fn metrics<C>(
	obj: &Text,
	text: &str,
	cache: &mut C
) -> f64 where C: CharacterCache {
	text.chars().fold(0.0, |sum, char| {
		sum + cache.character(obj.font_size, char).unwrap().advance_width()
	})
}

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
	fn render(self: &Self, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device);

	fn process_action(self: &Self, communications: &mut Communications, action: GameAction) -> Option<Box<dyn Player + Send>>;
}

enum Drawer {
	PickingWord(Vec<String>),
	Drawing(String, f64, u32),
	Done(bool),
}

const WORDS: &'static [&'static str] = &[
	"bike", "snowman", "tree", "flower", "basketball",
	"mountain", "turtle", "book", 
];

impl Drawer {
	fn new() -> Self {
		let words: Vec<String> = WORDS
			.choose_multiple(&mut rand::thread_rng(), 3)
			.map(|word| String::from(*word))
			.collect();

		Drawer::PickingWord(words)
	}
}

impl Player for Drawer {
	fn render(self: &Self, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, _device: &mut gfx_device_gl::Device) {
		match self {
			Drawer::PickingWord(words) => {
				center_text(font, glyphs, "Pick Word", 400.0, 50.0, c, g);
				for i in 0..words.len() {
					center_text(font, glyphs, &format!("[{}] {}", i + 1, words[i]), 400.0, 100.0 + 50.0 * (i as f64), c, g);
				}
			},

			Drawer::Drawing(word, _dt, time) => {
				center_text(font, glyphs, &format!("Drawing '{}'", word), 400.0, 50.0, c, g);
				
				font.draw(
					&time.to_string(),
					glyphs,
					&c.draw_state,
					c.transform.trans(10.0, 30.0), g
				).unwrap();
			},

			Drawer::Done(won) => {
				if *won {
					center_text(font, glyphs, "You Win", 400.0, 150.0, c, g);
				} else {
					center_text(font, glyphs, "Time's Up", 400.0, 150.0, c, g);
				}

				center_text(font, glyphs, "[y] Play Again?", 400.0, 250.0, c, g);
			}
		}
	}

	fn process_action(self: &Self, communications: &mut Communications, action: GameAction) -> Option<Box<dyn Player + Send>> {
		if let GameAction::SwapRoles = action {
			return Some(Box::new(Guesser::new()))
		}

		match self {
			Drawer::PickingWord(words) => {
				match action {
					GameAction::TypeNumber(n) => {
						let n = n as usize;
						if n > 0 && n <= words.len() {
							let word = words[n - 1].clone();
							let skeleton: String = word.chars().map(|c| {
								match c {
									' ' => ' ',
									_ => '_'
								}
							}).collect();
							GameMessage::SetWordSkeleton(skeleton).send(&mut communications.stream);

							Some(Box::new(Drawer::Drawing(word, 0.0, DRAWING_TIME)))
						} else {
							None
						}
					},

					_ => None
				}
			},

			Drawer::Drawing(word, cdt, time) => {
				match action {
					GameAction::LeftClick(x, y) => {
						GameMessage::Draw(x, y).send(&mut communications.stream);
						communications.action_sender.send(GameAction::Draw(x, y)).unwrap();
						None
					},

					GameAction::RightClick(x, y) => {
						GameMessage::Erase(x, y).send(&mut communications.stream);
						communications.action_sender.send(GameAction::Erase(x, y)).unwrap();
						None
					},

					GameAction::Update(dt) => {
						let cdt = cdt + dt;
						if cdt > 1.0 {
							communications.action_sender.send(GameAction::Tick).unwrap();
							None
						} else {
							Some(Box::new(Drawer::Drawing(word.clone(), cdt, *time)))
						}
					},

					GameAction::Tick => {
						let time = cmp::max(time - 1, 0);
						GameMessage::SetTimeRemaining(time).send(&mut communications.stream);
						
						if time > 0 {
							Some(Box::new(Drawer::Drawing(word.clone(), 0.0, time)))
						} else {
							GameMessage::GameOver(word.clone()).send(&mut communications.stream);
							Some(Box::new(Drawer::Done(false)))
						}
					},

					GameAction::Guess(guess) => {
						if guess == *word {
							GameMessage::GuessResult(Some(word.clone())).send(&mut communications.stream);
							Some(Box::new(Drawer::Done(true)))
						} else {
							GameMessage::GuessResult(None).send(&mut communications.stream);
							None
						}
					},

					_ => None
				}
			},

			Drawer::Done(_) => {
				match action {
					GameAction::TypeLetter(char) => {
						if char == 'y' {
							communications.action_sender.send(GameAction::SwapRoles).unwrap();
							GameMessage::SwapRoles.send(&mut communications.stream);
						}

						None
					},

					_ => None
				}
			}
		}
		
	}
}

enum Guesser {
	WaitingForDrawer,
	Guessing(u32, String, String),
	Done(bool, String),
}

impl Guesser {
	fn new() -> Self {
		Guesser::WaitingForDrawer
	}
}

impl Player for Guesser {
	fn render(self: &Self, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, _device: &mut gfx_device_gl::Device) {
		match self {
			Guesser::WaitingForDrawer => {
				center_text(font, glyphs, "Waiting for Drawer", 400.0, 50.0, c, g);
			},

			Guesser::Guessing(time, skeleton, guess) => {
				let guess: String = skeleton.chars().enumerate().map(|(index, skel_char)| {
					if let Some(guess_char) = guess.chars().nth(index) {
						guess_char
					} else {
						skel_char
					}
				}).collect();

				center_text(font, glyphs, &guess, 400.0, 50.0, c, g);

				font.draw(
					&time.to_string(),
					glyphs,
					&c.draw_state,
					c.transform.trans(10.0, 30.0), g
				).unwrap();
			},

			Guesser::Done(did_win, word) => {
				if *did_win {
					center_text(font, glyphs, "You Win", 400.0, 150.0, c, g);
				} else {
					center_text(font, glyphs, "Time's Up", 400.0, 150.0, c, g);
				}

				center_text(font, glyphs, &format!("'{}'", word), 400.0, 250.0, c, g);
			}
		};
	}

	fn process_action(self: &Self, communications: &mut Communications, action: GameAction) -> Option<Box<dyn Player + Send>> {
		if let GameAction::SwapRoles = action {
			return Some(Box::new(Drawer::new()))
		}

		match self {
			Guesser::WaitingForDrawer => {
				match action {
					GameAction::SetWordSkeleton(skeleton) => {
						Some(Box::new(Guesser::Guessing(DRAWING_TIME, skeleton, "".to_owned())))
					}

					_ => None
				}
			},

			Guesser::Guessing(t, skeleton, guess) => {
				match action {
					GameAction::SetTimeRemaining(time) => {
						Some(Box::new(Guesser::Guessing(time, skeleton.clone(), guess.clone())))
					},

					GameAction::GameOver(word) => {
						Some(Box::new(Guesser::Done(false, word.clone())))
					},

					GameAction::TypeLetter(char) => {
						if guess.len() < skeleton.len() {
							let mut new_guess = guess.clone();
							new_guess.push(char);
							
							Some(Box::new(Guesser::Guessing(*t, skeleton.clone(), new_guess)))
						} else {
							None
						}
					},

					GameAction::DeleteLetter => {
						if guess.len() > 0 {
							let mut new_guess = guess.clone();
							new_guess.pop();
							
							Some(Box::new(Guesser::Guessing(*t, skeleton.clone(), new_guess)))
						} else {
							None
						}
					},

					GameAction::Enter => {
						if guess.len() == skeleton.len() {
							GameMessage::Guess(guess.clone()).send(&mut communications.stream);
						}

						None
					},

					GameAction::GuessResult(res) => {
						if let Some(word) = res {
							Some(Box::new(Guesser::Done(true, word.clone())))
						} else {
							Some(Box::new(Guesser::Guessing(*t, skeleton.clone(), "".to_owned())))
						}
					},

					_ => None
				}
			},

			_ => None
		}
	}
}

struct WaitingPlayer {}

impl Player for WaitingPlayer {
	fn render(self: &Self, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, _device: &mut gfx_device_gl::Device) {
		center_text(font, glyphs, "Connecting...", 400.0, 50.0, c, g);
	}

	fn process_action(self: &Self, _communications: &mut Communications, _action: GameAction) -> Option<Box<dyn Player + Send>> { None }
}

struct EventState {
	left_mouse_down: bool,
	right_mouse_down: bool,
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

enum KeyboardButtonType {
	Letter(char),
	Number(u8),
	Enter,
	Backspace,
	Other
}

impl Game {
	pub fn new(canvas_op_sender: Sender<CanvasOperation>) -> Arc<Mutex<Self>> {
		let (sender, receiver) = channel();
		let this = Arc::new(Mutex::new(Game {
			communications: None,
			role: Box::new(WaitingPlayer {}),
			event_state: EventState {
				left_mouse_down: false,
				right_mouse_down: false,
			}
		}));

		let connection_thread_ref = this.clone();
		thread::spawn(move || {
			let args: Vec<String> = env::args().collect();
			let (stream, role): (TcpStream, Box<dyn Player + Send>) = if args.len() >= 2 {
				let ip = args.get(1).unwrap();
				println!("connecting to {}...", ip);
				(TcpStream::connect(ip).unwrap(), Box::new(Guesser::new()))
			} else {
				println!("waiting for connection...");
				(TcpListener::bind("127.0.0.1:4912").unwrap().accept().unwrap().0, Box::new(Drawer::new()))
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
				action_thread_ref.lock().unwrap().process_action(action);
			}
		});	

		this
	}

	fn translate_message(message: GameMessage) -> GameAction {
		match message {
			GameMessage::Draw(x, y) => GameAction::Draw(x, y),
			GameMessage::SetTimeRemaining(t) => GameAction::SetTimeRemaining(t),
			GameMessage::SetWordSkeleton(skeleton) => GameAction::SetWordSkeleton(skeleton),
			GameMessage::Guess(guess) => GameAction::Guess(guess),
			GameMessage::GuessResult(res) => GameAction::GuessResult(res),
			GameMessage::GameOver(word) => GameAction::GameOver(word),
			GameMessage::SwapRoles => GameAction::SwapRoles,
			GameMessage::Erase(x, y) => GameAction::Erase(x, y),
		}
	}

	pub fn process_action(&mut self, action: GameAction) {
		if let Some(communications) = &mut self.communications {
			match action {
				GameAction::Draw(x, y) => {
					communications.canvas_op_sender.send(CanvasOperation::Pixel(x, y, 0, 0, 255)).unwrap();
				},

				GameAction::Erase(x, y) => {
					communications.canvas_op_sender.send(CanvasOperation::Erase(x, y)).unwrap();
				},

				GameAction::SwapRoles => {
					communications.canvas_op_sender.send(CanvasOperation::Clear).unwrap()
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
			if let Some(p) = e.mouse_cursor_args() {
				let x = (p[0] as u32) / 8;
				let y = (p[1] as u32) / 8;

				if self.event_state.left_mouse_down {
					communications.action_sender.send(GameAction::LeftClick(x, y)).unwrap();
				} 

				if self.event_state.right_mouse_down {
					communications.action_sender.send(GameAction::RightClick(x, y)).unwrap();
				} 
			}

			match e {
				Event::Input(input, _) => {
					if let Input::Button(args) = input {
						match args.button {
							Button::Keyboard(key) => {
								if let ButtonState::Press = args.state {
									let keyboard_button: KeyboardButtonType = match key {
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
									};
							
									match keyboard_button {
										KeyboardButtonType::Letter(char) => {
											communications.action_sender.send(GameAction::TypeLetter(char)).unwrap();
										},

										KeyboardButtonType::Number(num) => {
											communications.action_sender.send(GameAction::TypeNumber(num)).unwrap();
										},

										KeyboardButtonType::Enter => {
											communications.action_sender.send(GameAction::Enter).unwrap();
										},

										KeyboardButtonType::Backspace => {
											communications.action_sender.send(GameAction::DeleteLetter).unwrap();
										},
										_ => {}
									};
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
									},

									MouseButton::Right => {
										self.event_state.right_mouse_down = state;
									},

									_ => {}
								};
							},
							_ => {}
						}
					};
				},

				Event::Loop(loop_event) => {
					if let Loop::Update(update_args) = loop_event {
						communications.action_sender.send(GameAction::Update(update_args.dt)).unwrap();
					}
				}
				_ => {},
			};
		}
	}

	pub fn render(&self, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device) {
		self.role.render(font, glyphs, c, g, device);
	}
}

