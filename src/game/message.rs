use std::io::{Read, Write};
use std::net::TcpStream;

const MESSAGE_DATA_SIZE: &'static [usize] = &[
	8, // draw
	4, // set time remaining
	4, // set word skeleton
	4, // guess
	5, // guess result
	4, // game over
	0, // swap roles
	8, // erase
	16, // draw line
	16, // erase line
];

pub enum GameMessage {
	Draw(u32, u32),
	SetTimeRemaining(u32),
	SetWordSkeleton(String),
	Guess(String),
	GuessResult(Option<String>),
	GameOver(String),
	SwapRoles,
	Erase(u32, u32),
	DrawLine(u32, u32, u32, u32),
	EraseLine(u32, u32, u32, u32),
}

pub fn parse_game_message(stream: &mut TcpStream) -> GameMessage {
	let mut id = [0u8; 1];
	stream.read_exact(&mut id).unwrap();

	let id = id[0] as usize;
	if id >= MESSAGE_DATA_SIZE.len() {
		panic!("unexpected message id, {}", id);
	}

	let s = MESSAGE_DATA_SIZE[id];
	let mut bytes = vec![0; s];
	if s > 0 {
		stream.read_exact(&mut bytes[..s]).unwrap();
	}

	match id {
		0 => {
			GameMessage::Draw(
				u32_from_bytes(&bytes[0..4]), 
				u32_from_bytes(&bytes[4..8])
			)
		}

		1 => {
			GameMessage::SetTimeRemaining(
				u32_from_bytes(&bytes[0..4])
			)
		},

		2 => {
			GameMessage::SetWordSkeleton(read_string(u32_from_bytes(&bytes[0..4]) as usize, stream))
		},

		3 => {	
			GameMessage::Guess(read_string(u32_from_bytes(&bytes[0..4]) as usize, stream))
		},

		4 => {
			let success = bytes[0] != 0;
			GameMessage::GuessResult(if success {
				Some(read_string(u32_from_bytes(&bytes[1..5]) as usize, stream))
			} else {
				None
			})
		},

		5 => {
			GameMessage::GameOver(read_string(u32_from_bytes(&bytes[0..4]) as usize, stream))
		},

		6 => GameMessage::SwapRoles,

		7 => GameMessage::Erase(
			u32_from_bytes(&bytes[0..4]), 
			u32_from_bytes(&bytes[4..8])
		),

		8 => GameMessage::DrawLine(
			u32_from_bytes(&bytes[0..4]), 
			u32_from_bytes(&bytes[4..8]),
			u32_from_bytes(&bytes[8..12]),
			u32_from_bytes(&bytes[12..16])
		),

		9 => GameMessage::EraseLine(
			u32_from_bytes(&bytes[0..4]), 
			u32_from_bytes(&bytes[4..8]),
			u32_from_bytes(&bytes[8..12]),
			u32_from_bytes(&bytes[12..16])
		),

		_ => panic!()
	}
}

impl GameMessage {
	fn id(&self) -> u8 {
		match &self {
			GameMessage::Draw(_, _) => 0,
			GameMessage::SetTimeRemaining(_) => 1,
			GameMessage::SetWordSkeleton(_) => 2,
			GameMessage::Guess(_) => 3,
			GameMessage::GuessResult(_) => 4,
			GameMessage::GameOver(_) => 5,
			GameMessage::SwapRoles => 6,
			GameMessage::Erase(_, _) => 7,
			GameMessage::DrawLine(_, _, _, _) => 8,
			GameMessage::EraseLine(_, _, _, _) => 9,
		}
	}

	pub fn send(&self, stream: &mut TcpStream) {
		let mut bytes = vec![self.id()];
		let push_u32 = |bytes: &mut Vec<u8>, i: u32| {
			bytes.extend_from_slice(&u32_to_bytes(i));
		};

		let push_string = |bytes: &mut Vec<u8>, s: &String| {
			let str_bytes = s.as_bytes();
			bytes.extend_from_slice(&u32_to_bytes(str_bytes.len() as u32));
			bytes.extend_from_slice(&str_bytes[..]);
		};

		match self {
			GameMessage::Draw(x, y) => {
				for v in [x, y] {
					push_u32(&mut bytes, *v);
				}
			},

			GameMessage::SetTimeRemaining(t) => {
				push_u32(&mut bytes, *t);
			},

			GameMessage::SetWordSkeleton(str) => {
				push_string(&mut bytes, str);
			},

			GameMessage::Guess(str) => {
				push_string(&mut bytes, str);
			},

			GameMessage::GuessResult(res) => {
				if let Some(word) = res {
					bytes.push(1);
					push_string(&mut bytes, word);
				} else {
					bytes.push(0);
					push_u32(&mut bytes, 0);
				}
			},

			GameMessage::GameOver(str) => {
				push_string(&mut bytes, str);
			},

			GameMessage::Erase(x, y) => {
				for v in [x, y] {
					push_u32(&mut bytes, *v);
				}
			},

			GameMessage::DrawLine(x1, y1, x2, y2) => {
				for v in [x1, y1, x2, y2] {
					push_u32(&mut bytes, *v);
				}
			},

			GameMessage::EraseLine(x1, y1, x2, y2) => {
				for v in [x1, y1, x2, y2] {
					push_u32(&mut bytes, *v);
				}
			},

			_ => {},
		}

		stream.write(&bytes[..]).unwrap();
	}
}

fn read_string(len: usize, stream: &mut TcpStream) -> String {
	let mut bytes = vec![0; len];
	stream.read_exact(&mut bytes[..len]).unwrap();

	String::from_utf8(bytes).unwrap()
}

fn u32_to_bytes(x: u32) -> [u8; 4] {
    [
		((x >> 24) & 0xff) as u8,
		((x >> 16) & 0xff) as u8,
		((x >> 8) & 0xff) as u8,
		(x & 0xff) as u8
	]
}

fn u32_from_bytes(bytes: &[u8]) -> u32 {
	let mut x: u32 = 0;
	x |= (bytes[0] as u32) << 24;
	x |= (bytes[1] as u32) << 16;
	x |= (bytes[2] as u32) << 8;
	x |= bytes[3] as u32;
	x
}
