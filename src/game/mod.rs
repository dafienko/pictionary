extern crate piston_window;
extern crate image as im;

mod communications;
mod game_action;
mod message;
mod text_util;
mod player;

use std::sync::mpsc::{channel, Sender};
use std::{env, thread};
use std::sync::{Mutex, Arc};
use std::net::{TcpStream, TcpListener};
use piston_window::*;
use game_action::GameAction;
use communications::Communications;
use crate::canvas::CanvasOperation;
use crate::game::message::{GameMessage, parse_game_message};
use crate::game::text_util::Glyphs;
use player::{Player, guesser::Guesser, drawer::Drawer, waiting_player::WaitingPlayer};

pub struct Game {
	role: Box<dyn Player + Send>,
	communications: Communications,
}

impl Game {
	pub fn new(canvas_op_sender: Sender<CanvasOperation>) -> Arc<Mutex<Self>> {
		let args: Vec<String> = env::args().collect();
		if args.len() < 2 {
			panic!("Not enough arguments provided (usage: address [is_host]")
		}

		let address = args[1].clone();
		let hosting = args.len() > 2;

		let (sender, receiver) = channel();
		let this = Arc::new(Mutex::new(Game {
			role: Box::new(WaitingPlayer::new(address.clone())),
			communications: Communications::new(None, sender.clone(), canvas_op_sender),
		}));

		let connection_thread_ref = this.clone();
		thread::spawn(move || {
			let (stream, role): (TcpStream, Box<dyn Player + Send>) = if hosting {
				(TcpListener::bind(address).unwrap().accept().unwrap().0, Box::new(Drawer::new()))
			} else {
				(TcpStream::connect(address).unwrap(), Box::new(Guesser::new()))
			};
			
			let action_sender = sender.clone();
			let mut reader = stream.try_clone().unwrap();
			connection_thread_ref.lock().unwrap().communications.set_stream(stream);

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
		match action {
			GameAction::Draw(x, y) => {
				self.communications.send_canvas_op(CanvasOperation::Pixel(x, y, 0, 0, 255));
			},

			GameAction::Erase(x, y) => {
				self.communications.send_canvas_op(CanvasOperation::Erase(x, y));
			},

			GameAction::SwapRoles => {
				self.communications.send_canvas_op(CanvasOperation::Clear);
			}
			_ => {}
		};
		
		if let Some(new) = self.role.process_action(&mut self.communications, action) {
			self.role = new;
		}
	}

	pub fn process_event(&mut self, e: Event) {
		self.communications.process_event(e)
	}

	pub fn render(&self, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device) {
		self.role.render(font, glyphs, c, g, device);
	}
}