extern crate piston_window;
extern crate image as im;

mod ts_dequeue;
mod text_util;
mod game;
mod message;
mod canvas;

use piston_window::*;
use std::{env, thread, vec};
use std::sync::{Arc, Mutex};
use std::net::{TcpStream, TcpListener};

use crate::canvas::GameCanvas;
use crate::game::Game;
use crate::text_util::metrics;

fn add_bytes(vec: &mut Vec<u8>, x: u32) {
    vec.push(((x >> 24) & 0xff) as u8);
	vec.push(((x >> 16) & 0xff) as u8);
	vec.push(((x >> 8) & 0xff) as u8);
	vec.push((x & 0xff) as u8);
}

fn parse_bytes(bytes: &[u8]) -> u32 {
	let mut x: u32 = 0;
	x |= (bytes[0] as u32) << 24;
	x |= (bytes[1] as u32) << 16;
	x |= (bytes[2] as u32) << 8;
	x |= bytes[3] as u32;
	x
}

enum DrawerState {
	Init,
	ChoosingWord(Vec<String>),
	Drawing(String)
}

enum GuesserState {
	Init,
	Waiting,
	Guessing(String)
}

enum Role {
	Drawer(DrawerState),
	Guesser(GuesserState)
}

fn connect_to_peer() -> (TcpStream, Role) {
	let args: Vec<String> = env::args().collect();
	if args.len() >= 2 {
		let ip = args.get(1).unwrap();
		println!("connecting to {}...", ip);
		(TcpStream::connect(ip).unwrap(), Role::Drawer(DrawerState::Init))
	} else {
		println!("waiting for connection...");
		(TcpListener::bind("127.0.0.1:4912").unwrap().accept().unwrap().0, Role::Guesser(GuesserState::Init))
	}
}

fn main() {
	let game = Game::new();
	println!("connected");

    let size = 100;
    let mut window: PistonWindow = WindowSettings::new(
            "piston: hello_world",
            [size * 4; 2]
        )
        .exit_on_esc(true)
        .graphics_api(OpenGL::V4_1)
        .build()
        .unwrap();

	let canvas = GameCanvas::new(&window, size, size);

	thread::spawn(|| { 
		canvas.process_operation_queue();
	});

	thread::spawn(|| {
		game.start_message_listener();
	});

	let mut mouse_down = false;
    while let Some(e) = window.next() {
        if e.render_args().is_some() {
            texture.update(&mut texture_context, &canvas.lock().unwrap()).unwrap();
            window.draw_2d(&e, |c, g, device| {
                

				
            });
        }

		if let Event::Input(input, _) = e.clone() {
			if let Input::Button(args) = input {
				if let Button::Mouse(mouse_button) = args.button {
					if let MouseButton::Left = mouse_button {
						mouse_down = match args.state {
							ButtonState::Press => true,
							ButtonState::Release => false,
						}
					}
				}
			}
		}

		if mouse_down {
			if let Some(p) = e.mouse_cursor_args() {
				if let Role::Drawer(_) = my_role {
					let x = p[0] as u32;
					let y = p[1] as u32;
					op_queue.push([x, y, 255, 0, 0]);

					let mut bytes = vec![];
					add_bytes(&mut bytes, x);
					add_bytes(&mut bytes, y);
					stream.write(&bytes[..]).unwrap();
				};
			}
		}
    }
}