extern crate piston_window;
extern crate image as im;

mod ts_dequeue;
mod text_util;
mod game;
mod message;
mod canvas;

use piston_window::*;
use crate::canvas::GameCanvas;
use crate::game::Game;

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

	let mut canvas = GameCanvas::new(&mut window, size, size);

    while let Some(e) = window.next() {
        if e.render_args().is_some() {
            canvas.pre_render();

            window.draw_2d(&e, |c, g, device| {
				canvas.render(c, g, device);
                game.lock().unwrap().render(&mut canvas, c, g, device);				
            });
        }

		game.lock().unwrap().process_event(e)
    }
}