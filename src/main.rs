extern crate piston_window;
extern crate image as im;

mod ts_dequeue;
mod text_util;

use piston_window::*;
use ts_dequeue::TSDequeue;
use std::{env, thread, vec};
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::net::{TcpStream, TcpListener};

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
	let stream = connect_to_peer();
	let mut my_role = stream.1;
	let mut stream = stream.0;
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

	let op_queue = Arc::new(TSDequeue::<[u32; 5]>::new());
    let canvas = Arc::new(Mutex::new(
		im::ImageBuffer::new(size, size)
	));
   
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    
	let mut texture: G2dTexture = Texture::from_image(
        &mut texture_context,
        &canvas.lock().unwrap(),
        &TextureSettings::new().filter(Filter::Nearest)
    ).unwrap();

	let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    println!("{:?}", assets);
    let mut glyphs = window.load_font(assets.join("FiraSans-Regular.ttf")).unwrap();

	let q = op_queue.clone();
	let c = canvas.clone();
	thread::spawn(move || { // render instructions
		loop {
			if !q.is_empty() {
				let [x, y, r, g, b] = q.pop();
				c.lock().unwrap().put_pixel(
					x / 4, y / 4, 
					im::Rgba([r as u8, g as u8, b as u8, 255])
				);
			}
		}
	});

	let mut s = stream.try_clone().unwrap();
	let sq = op_queue.clone();
	thread::spawn(move || { // read instructions from tcp stream
		let mut buf = vec![0; 8];
		loop {
			s.read_exact(&mut buf[0..8]).unwrap();
			let x = parse_bytes(&buf[0..4]);
			let y = parse_bytes(&buf[4..8]);
			sq.push([x, y, 0, 0, 255]);
		}
	});

	let font = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 32);
	let mut mouse_down = true;
    while let Some(e) = window.next() {
        if e.render_args().is_some() {
            texture.update(&mut texture_context, &canvas.lock().unwrap()).unwrap();
            window.draw_2d(&e, |c, g, device| {
                texture_context.encoder.flush(device);

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

				match my_role {
					Role::Drawer(_) => {
						center_text("Pick Word", size as f64 * 2.0, 50.0);

						center_text("[1] test", size as f64 * 2.0, 100.0);
						center_text("[2] word", size as f64 * 2.0, 140.0);
						center_text("[3] token", size as f64 * 2.0, 180.0);
					},
					Role::Guesser(_) => {
						center_text("Guess Word", size as f64 * 2.0, 50.0);
					}
				}

				// Update glyphs before rendering.
				glyphs.factory.encoder.flush(device);
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