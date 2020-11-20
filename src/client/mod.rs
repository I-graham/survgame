pub mod types;
pub mod state;

use super::utils;
use std::net;

pub fn client(address : &str) {
	use std::str::FromStr;

	let server_addr;
	if let Ok(addr) = net::SocketAddr::from_str(address) {
		server_addr = addr;
	} else if let Ok(addr) = net::IpAddr::from_str(address) {
		server_addr = net::SocketAddr::new(addr, utils::SERVER_PORT);
	} else {
		server_addr = net::SocketAddr::new("127.0.0.1".parse().unwrap(), utils::SERVER_PORT);
	}

	let event_loop = winit::event_loop::EventLoop::new();
	let mut game_state = state::ClientGame::new(server_addr, None, None, &event_loop);

	event_loop.run(move |event, _, control_flow| {

		use winit::event::*;
		use winit::event_loop::ControlFlow;
		match event {

			Event::WindowEvent {
				event,
				window_id,
			} if window_id == game_state.win_state.id() => {

				match event {

					WindowEvent::CloseRequested => {
						*control_flow = ControlFlow::Exit;
					},

					WindowEvent::Resized(dims) if dims.height != 0 && dims.width != 0 => {
						game_state.resize(dims);
					},

					WindowEvent::KeyboardInput { input, ..} => game_state.win_state.capture_key(input),

					WindowEvent::CursorMoved {
						position,
						..
					} => { game_state.win_state.capture_mouse(&position)},

					_ => {},

				}
			},

			Event::MainEventsCleared => {
				game_state.run();
				game_state.win_state.window.request_redraw();
			},

			Event::RedrawRequested(id) if id == game_state.win_state.id() => {
				game_state.draw();
			},

			_ => {},

		}
	});
}
