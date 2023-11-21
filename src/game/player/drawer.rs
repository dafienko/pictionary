use std::cmp;
use rand::seq::SliceRandom;
use piston_window::*;
use crate::game::game_action::GameAction;
use crate::game::communications::Communications;
use crate::game::message::GameMessage;
use crate::game::text_util::{Glyphs, *};
use crate::game::player::{Player, DRAWING_TIME, guesser::Guesser};

pub enum Drawer {
	PickingWord(Vec<String>),
	Drawing(String, f64, u32),
	Done(bool),
}

const WORDS: &'static [&'static str] = &[
	"bike", "snowman", "tree", "flower", "basketball",
	"mountain", "turtle", "book", 
];

impl Drawer {
	pub fn new() -> Self {
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
							communications.send_message(GameMessage::SetWordSkeleton(skeleton));

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
						communications.send_message(GameMessage::Draw(x, y));
						communications.send_action(GameAction::Draw(x, y));
						None
					},

					GameAction::RightClick(x, y) => {
						communications.send_message(GameMessage::Erase(x, y));
						communications.send_action(GameAction::Erase(x, y));
						None
					},

					GameAction::Update(dt) => {
						let cdt = cdt + dt;
						if cdt > 1.0 {
							communications.send_action(GameAction::Tick);
							None
						} else {
							Some(Box::new(Drawer::Drawing(word.clone(), cdt, *time)))
						}
					},

					GameAction::Tick => {
						let time = cmp::max(time - 1, 0);
						communications.send_message(GameMessage::SetTimeRemaining(time));
						
						if time > 0 {
							Some(Box::new(Drawer::Drawing(word.clone(), 0.0, time)))
						} else {
							communications.send_message(GameMessage::GameOver(word.clone()));
							Some(Box::new(Drawer::Done(false)))
						}
					},

					GameAction::Guess(guess) => {
						if guess == *word {
							communications.send_message(GameMessage::GuessResult(Some(word.clone())));
							Some(Box::new(Drawer::Done(true)))
						} else {
							communications.send_message(GameMessage::GuessResult(None));
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
							communications.send_action(GameAction::SwapRoles);
							communications.send_message(GameMessage::SwapRoles);
						}

						None
					},

					_ => None
				}
			}
		}
		
	}
}