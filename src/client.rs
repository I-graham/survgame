use std::net;

use crate::common::*;

pub fn client() {

	let addr = net::SocketAddr::new(get_public_ip(), SERVER_PORT);

	let stream = net::TcpStream::connect(addr).expect("unable to connect.");
	println!("connected to {}", stream.peer_addr().unwrap())
}