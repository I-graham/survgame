use crate::reng;

pub fn client() {

	let event_loop = winit::event_loop::EventLoop::new();
	let mut game_state = ClientGameState::new(&event_loop);

	game_state.renderer.draw_test();

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
					}

					_ => {},

				}
			}

			_ => {},

		}
	});
}

const START_WIN_SIZE : winit::dpi::LogicalSize<f32> = winit::dpi::LogicalSize {
	width : 800.0,
	height : 600.0,
};


struct WinState {
	window       : winit::window::Window,
	win_size     : winit::dpi::PhysicalSize<u32>,
	mouse_pos    : (f64, f64),
	mouse_down_l : bool,
	key_w        : bool,
	key_a        : bool,
	key_s        : bool,
	key_d        : bool,
	shft_down    : bool,
	ctrl_down    : bool,
}

impl WinState {
	fn new(event_loop : &winit::event_loop::EventLoopWindowTarget<()>) -> Self {

		let window = winit::window::Window::new(event_loop).expect("unable to create window");
		let win_size = window.inner_size();

		Self {
			window,
			win_size,
			mouse_pos    : (0.0,0.0),
			mouse_down_l : false,
			key_w        : false,
			key_a        : false,
			key_s        : false,
			key_d        : false,
			shft_down    : false,
			ctrl_down    : false,

		}

	}

	fn id(&self) -> winit::window::WindowId {
		self.window.id()
	}
}

struct ClientGameState {
	win_state : WinState,
	renderer  : reng::Renderer2D,
}

impl ClientGameState {
	fn new(event_loop: &winit::event_loop::EventLoopWindowTarget<()>) -> Self {

		let win_state = WinState::new(event_loop);
		let renderer  = reng::Renderer2D::new(&win_state.window, 1);

		ClientGameState {
			win_state,
			renderer,
		}
	}
}
