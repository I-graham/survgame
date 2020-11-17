use std::env;

mod utils;
mod server;
mod client;
mod reng;
mod world;
mod comms;

fn main() {
	let args : Vec<String> = env::args().collect();
	let address = args.get(2).map(|s| s.as_str()).unwrap_or("127.0.0.1:8778");
	if args.len() >= 2 {
		match args[1].as_str() {
			"host" => {
				server::server();
			},
			"client" => {
				client::client(address);
			},
			"local" => {
				std::thread::spawn(move || {
					server::server();
				});
				client::client(address);
			},
			_ => {
				println!("Invalid argument, exiting process.");
			},
		}
	} else {
		client::client(address);
	}

}