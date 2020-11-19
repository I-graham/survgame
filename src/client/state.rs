use super::types;
use crate::reng;
use crate::reng::types::*;
use crate::utils;
use crate::world::World;
use crate::comms::{Action, Perception};

use std::net;
use std::collections::VecDeque;
use winit::event::VirtualKeyCode;
use std::hash::Hash;
use fnv::FnvHashMap;
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

			let coords = GLvec4(
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
	pub timestep       : utils::Timer,
	pub uniform        : types::Uniform,
	pub instance_queue : Vec<types::Instance2D>,
	pub action_queue   : VecDeque<(f32, Action)>,
	pub texture_map    : FnvHashMap<ClientTexture, GLvec4>,
	pub world          : World,
	pub stream         : net::TcpStream,
	pub id             : usize,
}

impl ClientGame {
	pub fn new(address : net::SocketAddr, vs_path : Option<&std::path::Path>, fs_path : Option<&std::path::Path>, event_loop: &winit::event_loop::EventLoopWindowTarget<()>,) -> Self {

		let win_state = types::WinState::new(event_loop);
		let mut renderer  = reng::Renderer2D::<types::Uniform, types::Instance2D>::new(&win_state.window, 1, vs_path, fs_path);

		let aspect = win_state.size.width as f32 / win_state.size.height as f32;
		let uniform = types::Uniform {
			ortho : cgmath::ortho(-aspect, aspect, -1., 1., -100., 100.),
		};

		let (spritesheet, texture_map) = ClientTexture::load_textures();

		let text = renderer.create_texture_from_image(&spritesheet);
		renderer.set_texture(&text);

		let action_queue = VecDeque::new();

		let instance_queue = vec![];

		let stream = net::TcpStream::connect(address).unwrap_or_else(
			|err| panic!("unable to connect to server at {:?}, due to the following error : {:?}", address, err)
		);

		let id;
		if let Perception::ID(player_id) = bincode::deserialize_from(&stream).expect("Unable to get data from server") {
			id = player_id as usize;
		} else {
			panic!("Unable to get ID from server.")
		}
		stream.set_nonblocking(true).expect("Unable to set nonblocking on TcpStream, that's odd...");
		stream.set_nodelay(true).expect("Unable to set nodelay on TcpStream, that's odd...");

		let timestep  = utils::Timer::new();
		ClientGame {
			win_state,
			renderer,
			timestep,
			uniform,
			texture_map,
			instance_queue,
			action_queue,
			stream,
			world : World::new(),
			id,
		}
	}

	pub fn draw(&mut self) {

		self.world.render_to(&mut self.instance_queue, &self.texture_map);

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
		self.generate_actions();

		while let Some(action) = self.action_queue.front_mut() {
			if action.0 < self.world.timestamp {
				self.action_queue.pop_front();
			}
		}

		for action in &self.action_queue {
			self.world.process(self.id, &action.1);
		}

		if let Ok(perception) = bincode::deserialize_from(&self.stream) {
			use Perception::*;
			match perception {
				World(world) => self.world = world,
				_ => (),
			}
		}

		self.world.update(self.timestep.secs());
		self.timestep.reset();
	}

	fn generate_actions(&mut self) {
		let turn_dir = *self.win_state.keymap.get(&VirtualKeyCode::A).unwrap_or(&false) as i32 - *self.win_state.keymap.get(&VirtualKeyCode::D).unwrap_or(&false) as i32;
		if turn_dir != 0 {
			let angle = turn_dir as f32 * 200.0 * self.timestep.secs();
			let action = Action::TurnShip(angle);
			self.world.process(self.id, &action);
			self.send_to_server(&action);
			let timestamp = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs_f32();
			self.action_queue.push_back((timestamp, action));
		}
	}

	fn send_to_server(&mut self, action : &Action) {
		bincode::serialize_into(&self.stream, &action).unwrap_or_else(|e| panic!("Unable to send data to server due to {:?}", e));
	}
}