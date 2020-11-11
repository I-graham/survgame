use std::net;
use std::thread;

use crate::utils;

pub fn server() {

	let server_ip = net::IpAddr::V4(net::Ipv4Addr::new(0,0,0,0));

	let socket_addr = net::SocketAddr::new(server_ip, utils::SERVER_PORT);

	let listener = net::TcpListener::bind(socket_addr).unwrap_or_else(|_| panic!("unable to listen on {}", socket_addr));

	let public_ip = utils::get_public_ip();

	println!("listening on {:?}", net::SocketAddr::new(public_ip, utils::SERVER_PORT));

	let mut clients : Vec<thread::JoinHandle<_>> = vec![];

	for stream in listener.incoming() {
		match stream {
			Ok(client) => {

				println!("New connection: {:?}", client);

				let join_handle = thread::spawn(move || {
					handle_client(client);
				});

				clients.push(join_handle);

			},
			Err(e) => {
				panic!("Error: {}", e);
			}
		}
	}

}

fn handle_client(mut client : net::TcpStream) {
	use std::io::Read;
	use std::str;

	let mut read_buf = [0u8; 512];
	while let Ok(len) = client.read(&mut read_buf) {
		println!("({}) : {}", len, str::from_utf8(&read_buf[..len]).unwrap());
	}
	println!("Connection closed with {:?}", client);

}