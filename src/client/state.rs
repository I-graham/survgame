use super::types;
use crate::reng;
use crate::reng::types::*;
use crate::utils;
use crate::world::World;
use crate::comms::*;

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
	pub action_queue   : VecDeque<TimestampedAction>,
	pub texture_map    : FnvHashMap<ClientTexture, GLvec4>,
	pub world          : World,
	pub server         : TypedStream<TimestampedAction, TimestampedPerception>,
	pub last_received  : f64,
	pub last_processed : f64,
	pub id             : usize,
}

impl ClientGame {
	pub fn new(address : net::SocketAddr, vs_path : Option<&std::path::Path>, fs_path : Option<&std::path::Path>, event_loop: &winit::event_loop::EventLoopWindowTarget<()>,) -> Self {

		let win_state = types::WinState::new(event_loop);
		let mut renderer  = reng::Renderer2D::<types::Uniform, types::Instance2D>::new(&win_state.window, 2, vs_path, fs_path);

		let aspect = win_state.size.width as f32 / win_state.size.height as f32;
		let uniform = types::Uniform {
			ortho : cgmath::ortho(-aspect, aspect, -1., 1., -100., 100.),
		};

		let (spritesheet, texture_map) = ClientTexture::load_textures();

		let text = renderer.create_texture_from_image(&spritesheet);
		renderer.set_texture(&text);

		let action_queue = VecDeque::new();

		let instance_queue = vec![];

		let stream = net::TcpStream::connect(address).expect("Unable to connect to server");

		let id;
		if let TimestampedPerception {
			perception : Perception::ID(player_id),
			..
		} = bincode::deserialize_from(&stream).expect("Unable to get data from server") {
			id = player_id as usize;
		} else {
			panic!("Unable to get ID from server.")
		}

		let world;
		let last_received;
		if let TimestampedPerception {
			perception : Perception::World(init_world),
			timestamp,
		} = bincode::deserialize_from(&stream).expect("Unable to get data from server") {
			world = init_world;
			last_received = timestamp;
		} else {
			panic!("Unable to get world state from server.")
		}

		let timestep  = utils::Timer::new();
		ClientGame {
			win_state,
			renderer,
			timestep,
			uniform,
			texture_map,
			instance_queue,
			action_queue,
			server : TypedStream::new(stream),
			world,
			id,
			last_received,
			last_processed : last_received,
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

		for action in &self.action_queue {
			if self.last_processed < action.timestamp {
				self.world.process(self.id, &action.action);
				self.last_processed = action.timestamp;
			}
		}

		self.world.update(self.timestep.secs());

		match self.server.recv() {
			Ok(ts_perc) => {
				use Perception::*;
				match ts_perc.perception {
					World(world) => {
						self.world = world;
						self.last_received = ts_perc.timestamp;
						self.last_processed = ts_perc.timestamp;
						while let Some(action) = self.action_queue.front() {
							if action.timestamp <= self.last_received {
								self.action_queue.pop_front();
							} else {
								break;
							}
						}

					},
					_ => (),
				}
			},

			Err(bincode::ErrorKind::Io(err)) if err.kind() == std::io::ErrorKind::WouldBlock => (),
			Err(bincode::ErrorKind::Io(err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => (),

			Err(err) => {
				panic!("{:?}", err);
			},
		}

		self.timestep.reset();
	}

	fn generate_actions(&mut self) {
		let player_ship = self.world.ships.get(self.id).unwrap();
		let turn_dir = *self.win_state.keymap.get(&VirtualKeyCode::A).unwrap_or(&false) as i8 - *self.win_state.keymap.get(&VirtualKeyCode::D).unwrap_or(&false) as i8;
		if turn_dir != player_ship.turning {
			let action = Action::TurnShip(turn_dir);
			let timestamp = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs_f64();
			let ts_act = TimestampedAction {
				timestamp,
				action,
			};
			self.action_queue.push_back(ts_act.clone());
			self.server.send(&ts_act);
		}
	}
}