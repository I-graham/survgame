mod state;

use crate::utils;
use crate::world;

pub fn server() {

	let mut server = state::Server::new();

	server.accept(1);

	server.world.ships.push(world::Ship::new());
	while server.process() {
	}

}

