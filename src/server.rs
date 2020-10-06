use std::net;

use crate::common::*;

pub fn server() {

	let server_ip = net::IpAddr::V4(net::Ipv4Addr::new(0,0,0,0));

	let socket_addr = net::SocketAddr::new(server_ip, SERVER_PORT);

	let listener = net::TcpListener::bind(socket_addr).expect(&format!("unable to listen on {}", socket_addr));

	println!("listening on {:?}", socket_addr);

	for stream in listener.incoming() {
		match stream {
			Ok(client) => {
				println!("New connection: {}", client.peer_addr().unwrap());
			},
			Err(e) => {
				panic!("Error: {}", e);
			}
		}
	}

}