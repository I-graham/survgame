mod types;

use crate::reng;
use std::collections::HashMap;

pub fn client() {

	let event_loop = winit::event_loop::EventLoop::new();
	let mut game_state = ClientGameState::new(&event_loop, None, None);

	let spritesheet = reng::utils::create_spritesheet(vec![
		image::open("assets/player.png").unwrap().to_rgba(),
	]).0;

	let text = game_state.renderer.create_texture_from_image(&spritesheet);
//	game_state.renderer.set_texture(&text);

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

const START_WIN_SIZE : winit::dpi::PhysicalSize<f32> = winit::dpi::PhysicalSize {
	width : 400.0,
	height : 400.0,
};


struct WinState {
	window       : winit::window::Window,
	size         : winit::dpi::PhysicalSize<u32>,
	aspect       : f32,
	mouse_pos    : (f32, f32),
	mouse_down_l : bool,
	keymap       : std::collections::HashMap<winit::event::VirtualKeyCode, bool>,
}

impl WinState {
	fn new(event_loop : &winit::event_loop::EventLoopWindowTarget<()>) -> Self {

		let window = winit::window::WindowBuilder::new()
			.with_min_inner_size(START_WIN_SIZE)
			.build(event_loop)
			.expect("unable to create window");

		let size = window.inner_size();
		let aspect = size.width as f32/ size.height as f32;

		Self {
			window,
			size,
			aspect,
			mouse_pos    : (0.0,0.0),
			mouse_down_l : false,
			keymap       : HashMap::new(),
		}

	}

	fn capture_mouse(&mut self, pos : &winit::dpi::PhysicalPosition<f64>) {
		self.mouse_pos = (
			2.0 * pos.x as f32 / self.size.width as f32 - 1.0,
			-2.0 * pos.y as f32 / self.size.height as f32 + 1.0
		);
	}

	fn capture_key(&mut self, input : winit::event::KeyboardInput) {
		use winit::event::{KeyboardInput, VirtualKeyCode, ElementState};
		let KeyboardInput { virtual_keycode : key, state, .. } = input;
		match key {
			Some(key) if (VirtualKeyCode::A..VirtualKeyCode::Z).contains(&key) => {self.keymap.insert(key, state == ElementState::Pressed);},
			_ => {},
		}
	}

	fn resize(&mut self, dims : winit::dpi::PhysicalSize<u32>) {
		self.size = dims;
		self.aspect = dims.width as f32 / dims.height as f32;
	}

	fn id(&self) -> winit::window::WindowId {
		self.window.id()
	}
}

struct ClientGameState {
	win_state : WinState,
	renderer  : reng::Renderer2D<types::Uniform>,
	timestep  : std::time::Instant,
	uniform   : types::Uniform,
}

impl ClientGameState {
	fn new(event_loop: &winit::event_loop::EventLoopWindowTarget<()>, vs_path : Option<&std::path::Path>, fs_path : Option<&std::path::Path>) -> Self {

		let win_state = WinState::new(event_loop);
		let renderer  = reng::Renderer2D::<types::Uniform>::new(&win_state.window, 1, vs_path, fs_path);
		let timestep  = std::time::Instant::now();

		let aspect = win_state.size.width as f32 / win_state.size.height as f32;
		let uniform = types::Uniform {
			aspect_ratio : cgmath::ortho(-aspect, aspect, -1., 1., -1., 1.),
		};

		ClientGameState {
			win_state,
			renderer,
			timestep,
			uniform,
		}
	}

	fn resize(&mut self, dims : winit::dpi::PhysicalSize<u32>) {
		self.renderer.resize(dims);
		self.win_state.resize(dims);
		self.uniform.aspect_ratio = cgmath::ortho(-self.win_state.aspect, self.win_state.aspect, -1., 1., -1., 1.);
	}

	fn run(&mut self) {
		let timestep = std::time::Instant::now().duration_since(self.timestep).as_secs_f32();
		self.timestep = std::time::Instant::now();

	}

	fn draw(&mut self) {
		let mut instances = &mut [
			reng::types::Instance2D {
				scale : reng::types::GLvec2(1.0, 1.0),
				translate : reng::types::GLvec2(-1.0, -1.0),
				color_tint : reng::types::GLvec4(0.0,0.0,0.0,0.0),
				texture_coords : reng::types::GLvec4(0.0, 0.0, 1.0, 1.0),
			},
		];

		instances[0].scale = reng::types::GLvec2(
			0.5,//400.0 / self.win_state.size.width as f32,
			0.5//400.0 / self.win_state.size.height as f32,
		);

		instances[0].translate = reng::types::GLvec2(
			self.win_state.mouse_pos.0 * self.win_state.aspect,
			self.win_state.mouse_pos.1
		);
		self.renderer.draw_test(&self.uniform, instances);
	}
}
