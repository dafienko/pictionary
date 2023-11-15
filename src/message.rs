use std::io::{prelude, Read};
use std::net::TcpStream;

use crate::parse_bytes;

pub enum GameMessage {
	PutPixel(u32, u32),
}

fn send_message(stream: TcpStream, message: GameMessage) {
	let id = message as isize;
	// let mut bytes = vec![id];
	match message {
		GameMessage::PutPixel(x, y) => {
			
		}
	}
}

fn u32_from_bytes(bytes: &[u8]) -> u32 {
	let mut x: u32 = 0;
	x |= (bytes[0] as u32) << 24;
	x |= (bytes[1] as u32) << 16;
	x |= (bytes[2] as u32) << 8;
	x |= bytes[3] as u32;
	x
}

fn parse_message(stream: &mut TcpStream) -> GameMessage {
	let mut id = [0u8; 1];
	stream.read_exact(&mut id).unwrap();

	match id[0] {
		0x00 => {
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