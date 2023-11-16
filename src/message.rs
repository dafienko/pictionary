use std::io::{prelude, Read, Write};
use std::net::TcpStream;

use crate::parse_bytes;

const PUT_PIXEL_MESSAGE_ID: u8 = 0x00u8;

pub enum GameMessage {
	PutPixel(u32, u32),
}

pub fn parse_game_message(stream: &mut TcpStream) -> GameMessage {
	let mut id = [0u8; 1];
	stream.read_exact(&mut id).unwrap();

	match id[0] {
		PUT_PIXEL_MESSAGE_ID => {
			let mut bytes = [0u8; 8];
			stream.read_exact(&mut bytes).unwrap();

			GameMessage::PutPixel(
				parse_bytes(&bytes[0..4]), 
				parse_bytes(&bytes[4..8])
			)
		}
		_ => panic!("unexpected message id, {}", id[0])
	}
}

impl GameMessage {
	fn id(&self) -> u8 {
		match &self {
			GameMessage::PutPixel(_, _) => PUT_PIXEL_MESSAGE_ID
		}
	}

	pub fn send(&self, stream: &mut TcpStream) {
		let mut bytes = vec![self.id()];
		match self {
			GameMessage::PutPixel(x, y) => {
				for v in [x, y] {
					bytes.extend_from_slice(&u32_to_bytes(*v));
				}
			}
		}

		stream.write(&bytes[..]).unwrap();
	}
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
