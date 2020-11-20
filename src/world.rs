use serde_derive::*;
use fnv::FnvHashMap;

use crate::comms;
use crate::reng::types::*;
use crate::client::types::Instance2D;
use crate::client::state::ClientTexture;


#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct World {
	pub ships : Vec<Ship>,
}

impl World {
	pub fn new() -> Self {
		Self {
			ships : vec![],
		}
	}

	pub fn process(&mut self, player_id : usize, action : &comms::Action) {
		use comms::Action::*;
		let player_ship = self.ships.get_mut(player_id).unwrap();
		match action {
			TurnShip(theta) => {
				player_ship.angle += theta;
			},
			_ => unimplemented!(),
		}
	}

	pub fn update(&mut self, timestep : f32) {
		for ship in &mut self.ships {
			ship.update(timestep);
		}
	}

	pub fn render_to(&self, output_buffer : &mut Vec<Instance2D>, texture_map : &FnvHashMap<ClientTexture, GLvec4>) {
		let ship_text = texture_map[&ClientTexture::Ship];
		output_buffer.extend(
			self.ships.iter().map(|ship| ship.render(ship_text))
		);
	}
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Ship {
	pub alive : bool,
	pub angle : f32,
	pub pos : (f32, f32),
	pub vel : (f32, f32),
	pub acc : (f32, f32),
}

impl Ship {

	pub fn new() -> Self {
		Self {
			alive : true,
			angle : 0.0,
			pos : (0.0,0.0),
			vel : (0.0,0.0),
			acc : (0.0,0.0),
		}
	}

	pub fn update(&mut self, timestep : f32) {
		self.vel.0 += self.acc.0 * timestep;
		self.vel.1 += self.acc.1 * timestep;

		self.pos.0 += self.vel.0 * timestep;
		self.pos.1 += self.vel.1 * timestep;
	}

	pub fn render(&self, text_coords : GLvec4) -> Instance2D {
		let mut instance = Instance2D::default();
		instance.texture_coords = text_coords;
		instance.translate = GLvec2(self.pos.0, self.pos.1);
		instance.rotation = GLfloat(self.angle - 90.0);
		instance
	}
}