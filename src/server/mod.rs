mod state;

use crate::utils;

pub fn server() {

	let mut server = state::Server::new();

	server.accept(2);

	while server.online() {
		server.process();
	}

}

