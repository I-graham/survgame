use crate::reng::types::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Uniform {
	pub ortho : cgmath::Matrix4<f32>,
}

#[repr(C, align(16))]
#[derive(Clone, Debug)]
pub struct Instance2D {
	pub color_tint     : GLvec4,
	pub texture_coords : GLvec4,
	pub scale          : GLvec2,
	pub translate      : GLvec2,
	pub rotation       : GLfloat,
	pub z_coord        : GLfloat,
}

impl Default for Instance2D {
	fn default() -> Self {
		Instance2D {
			color_tint : GLvec4(1.0,1.0,1.0,1.0),
			texture_coords : GLvec4(0.0,0.0,1.0,1.0),
			scale : GLvec2(1.0,1.0),
			translate : GLvec2(0.0,0.0),
			rotation : GLfloat(0.0),
			z_coord : GLfloat(0.0),
		}
	}
}

const START_WIN_SIZE : winit::dpi::PhysicalSize<f32> = winit::dpi::PhysicalSize {
	width : 400.0,
	height : 400.0,
};

pub struct WinState {
	pub window       : winit::window::Window,
	pub size         : winit::dpi::PhysicalSize<u32>,
	pub aspect       : f32,
	pub mouse_pos    : (f32, f32),
	pub mouse_down_l : bool,
	pub keymap       : fnv::FnvHashMap<winit::event::VirtualKeyCode, bool>,
}

impl WinState {
	pub fn new(event_loop : &winit::event_loop::EventLoopWindowTarget<()>) -> Self {

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
			keymap       : fnv::FnvHashMap::default(),
		}

	}

	pub fn capture_mouse(&mut self, pos : &winit::dpi::PhysicalPosition<f64>) {
		self.mouse_pos = (
			2.0 * pos.x as f32 / self.size.width as f32 - 1.0,
			-2.0 * pos.y as f32 / self.size.height as f32 + 1.0
		);
	}

	pub fn capture_key(&mut self, input : winit::event::KeyboardInput) {
		use winit::event::{KeyboardInput, VirtualKeyCode, ElementState};
		let KeyboardInput { virtual_keycode : key, state, .. } = input;
		match key {
			Some(key) if (VirtualKeyCode::A..VirtualKeyCode::Z).contains(&key) => {self.keymap.insert(key, state == ElementState::Pressed);},
			_ => {},
		}
	}

	pub fn resize(&mut self, dims : winit::dpi::PhysicalSize<u32>) {
		self.size = dims;
		self.aspect = dims.width as f32 / dims.height as f32;
	}

	pub fn id(&self) -> winit::window::WindowId {
		self.window.id()
	}
}