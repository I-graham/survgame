use serde_derive::*;
use std::net;
use crate::world;

#[derive(Serialize, Clone, Deserialize, Debug)]
pub enum Action {
	Disconnect,
	Message(String),
	TurnShip(f32),
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TimestampedAction {
	pub timestamp : f64,
	pub action : Action,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub enum Perception {
	ID(i32),
	World(world::World)
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TimestampedPerception {
	pub timestamp : f64,
	pub perception : Perception,
}

pub struct Client {
	pub stream : net::TcpStream,
	pub timestamp : f64,
	pub online : bool,
}

impl Client {
	pub fn new(stream : net::TcpStream) -> Self {
		Self {
			stream,
			timestamp : std::time::UNIX_EPOCH.elapsed().unwrap().as_secs_f64(),
			online : true,
		}
	}

	pub fn authorative_send_to(&self, perception : Perception) {
		let ts_perc = TimestampedPerception {
			timestamp : self.timestamp,
			perception,
		};
		bincode::serialize_into(&self.stream, &ts_perc).expect("unable to serialize world state.");
	}

	pub fn disconnect(&mut self) {
		self.online = false;
		let _ = self.stream.shutdown(net::Shutdown::Both);
	}

}