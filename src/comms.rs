use serde_derive::*;
use std::net;
use crate::world;

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
	TurnShip(f32),
	Disconnect,
	ID(i32),
	Message(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Perception {
	ID(i32),
	World(world::World)
}


pub struct Client {
	pub stream : net::TcpStream,
	pub online : bool,
}

impl Client {
	pub fn new(stream : net::TcpStream) -> Self {
		Self {
			stream,
			online : true,
		}
	}

	pub fn authorative_send_to(&self, perception : &Perception) {
		bincode::serialize_into(&self.stream, perception).expect("unable to serialize world state.");
	}

	pub fn disconnect(&mut self) {
		self.online = false;
		let _ = self.stream.shutdown(net::Shutdown::Both);
	}

}