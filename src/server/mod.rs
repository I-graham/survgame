mod state;

use crate::utils;

pub fn server() {

	let mut server = state::Server::new();

	server.accept(1);

	while server.process() {}

}

