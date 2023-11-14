extern crate piston_window;
extern crate image as im;

mod ts_dequeue;

use piston_window::*;
use ts_dequeue::TSDequeue;
use std::io::prelude::*;
use std::{env, thread, vec};
use std::sync::{Arc, Mutex};
use std::net::{TcpStream, TcpListener};

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

fn connect_to_peer() -> TcpStream {
	let args: Vec<String> = env::args().collect();
	let stream = if args.len() >= 2 {
		let ip = args.get(1).unwrap();
		println!("connecting to {}...", ip);
		TcpStream::connect(ip).unwrap()
	} else {
		println!("waiting for connection...");
		TcpListener::bind("127.0.0.1:4912").unwrap().accept().unwrap().0
	};
	
	println!("connected");
	stream
}

fn main() {
	let mut stream = connect_to_peer();
	
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

    while let Some(e) = window.next() {
        if e.render_args().is_some() {
            texture.update(&mut texture_context, &canvas.lock().unwrap()).unwrap();
            window.draw_2d(&e, |c, g, device| {
                texture_context.encoder.flush(device);

                clear([1.0; 4], g);
                image(&texture, c.transform.scale(4.0, 4.0), g);
            });
        }

        if let Some(p) = e.mouse_cursor_args() {
            let x = p[0] as u32;
            let y = p[1] as u32;
			op_queue.push([x, y, 255, 0, 0]);

			let mut bytes = vec![];
			add_bytes(&mut bytes, x);
			add_bytes(&mut bytes, y);
			stream.write(&bytes[..]).unwrap();
        }
    }
}