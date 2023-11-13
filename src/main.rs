extern crate piston_window;
extern crate image as im;

mod ts_dequeue;

use std::thread;
use std::sync::{Arc, Mutex};
use piston_window::*;
use ts_dequeue::TSDequeue;

fn main() {
    let size = 100;
    let mut window: PistonWindow = WindowSettings::new(
            "piston: hello_world",
            [size * 4; 2]
        )
        .exit_on_esc(true)
        .graphics_api(OpenGL::V4_1)
        .build()
        .unwrap();

	let op_queue = Arc::new(TSDequeue::<[u32; 2]>::new());
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

    window.set_lazy(true);

	let q = op_queue.clone();
	let c = canvas.clone();
	thread::spawn(move || {
		loop {
			if !q.is_empty() {
				let [x, y] = q.pop();
				c.lock().unwrap().put_pixel(x / 4, y / 4, im::Rgba([0, 0, 0, 255]));
			}
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

			op_queue.push([x, y]);
        }
    }
}