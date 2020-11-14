use super::reng;
use super::types;
use fnv::FnvHashMap;
use std::hash::Hash;
use strum::IntoEnumIterator;
use strum_macros::{IntoStaticStr, EnumIter};

#[derive(IntoStaticStr, EnumIter, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ClientTexture {
	Flat,
	Player,
	Ship,
}

impl ClientTexture {
	fn load_textures() -> (image::RgbaImage, FnvHashMap<ClientTexture, reng::types::GLvec4>) {
		let mut map = FnvHashMap::default();

		let mut rbga_images = Self::iter().map(|text| {
			let file_name = format!("assets/{}.png", <&'static str>::from(text));
			image::open(file_name).unwrap().into_rgba()
		}).collect::<Vec<_>>();

		let img_size = |img : &image::RgbaImage| (img.height() * img.width()) as i32;

		let mut sorted_iter = Self::iter().enumerate().collect::<Vec<_>>();
		sorted_iter.sort_by_key(|(index, _text)| {
			-img_size(&rbga_images[*index])
		});

		rbga_images.sort_by_key(|e| -img_size(e) );

		let spritesheet = reng::utils::create_spritesheet(rbga_images);

		let image_dims = spritesheet.0.dimensions();

		let pixel_to_text_coord = |(x, y)| {
			let norm_x = x as f32 / image_dims.0 as f32;
			let norm_y = y as f32 / image_dims.1 as f32;
			(norm_x, norm_y)
		};

		for (text, pos) in sorted_iter.iter().map(|(_index, text)| text).zip(&spritesheet.1) {
			let coord_ul = pixel_to_text_coord(pos.0);
			let coord_lr = pixel_to_text_coord(pos.1);

			let coords = reng::types::GLvec4(
				coord_ul.0,
				coord_ul.1,
				coord_lr.0,
				coord_lr.1,
			);

			map.insert(*text, coords);
		}

		(spritesheet.0, map)

	}
}

pub struct ClientGame {
	pub renderer       : reng::Renderer2D<types::Uniform, types::Instance2D>,
	pub win_state      : types::WinState,
	pub timestep       : std::time::Instant,
	pub uniform        : types::Uniform,
	pub instance_queue : Vec<types::Instance2D>,
	pub texture_map    : FnvHashMap<ClientTexture, reng::types::GLvec4>,
}

impl ClientGame {
	pub fn new(event_loop: &winit::event_loop::EventLoopWindowTarget<()>, vs_path : Option<&std::path::Path>, fs_path : Option<&std::path::Path>) -> Self {

		let win_state = types::WinState::new(event_loop);
		let timestep  = std::time::Instant::now();
		let mut renderer  = reng::Renderer2D::<types::Uniform, types::Instance2D>::new(&win_state.window, 1, vs_path, fs_path);

		let aspect = win_state.size.width as f32 / win_state.size.height as f32;
		let uniform = types::Uniform {
			ortho : cgmath::ortho(-aspect, aspect, -1., 1., -1., 1.),
		};

		let (spritesheet, texture_map) = ClientTexture::load_textures();

		let text = renderer.create_texture_from_image(&spritesheet);
		renderer.set_texture(&text);

		let instance_queue = vec![];

		ClientGame {
			win_state,
			renderer,
			timestep,
			uniform,
			texture_map,
			instance_queue,
		}
	}

	pub fn draw(&mut self) {

		let scale = reng::types::GLvec2(
			1.,
			1.,
		);

		let translate = reng::types::GLvec2(
			self.win_state.mouse_pos.0 * self.win_state.aspect,
			self.win_state.mouse_pos.1
		);

		let rotation = reng::types::GLfloat(0.0);

		let instance = types::Instance2D {
			color_tint     : reng::types::GLvec4(1.0,1.0,1.0,1.0),
			texture_coords : self.texture_map[&ClientTexture::Ship],
			scale,
			translate,
			rotation,
		};

		self.instance_queue.push(instance);

		let instances = self.instance_queue.as_slice();
		self.renderer.draw_test(&self.uniform, instances);
		self.instance_queue.clear();
	}

	pub fn resize(&mut self, dims : winit::dpi::PhysicalSize<u32>) {
		self.renderer.resize(dims);
		self.win_state.resize(dims);
		self.uniform.ortho = cgmath::ortho(-self.win_state.aspect, self.win_state.aspect, -1., 1., -1., 1.);
	}

	pub fn run(&mut self) {
		let timestep = std::time::Instant::now().duration_since(self.timestep).as_secs_f32();
		self.timestep = std::time::Instant::now();


	}
}
