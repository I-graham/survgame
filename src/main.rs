use std::env;

mod utils;
mod server;
mod client;
mod reng;

fn main() {
	let args : Vec<String> = env::args().collect();

	if args.len() >= 2 {
		match args[1].as_str() {
			"host" => {
				server::server();
			},
			"client" => {
				client::client();
			},
			_ => {
				println!("Invalid argument, exiting process.");
			},
		}
	} else {
		client::client();
	}

}

#[cfg(test)]
mod tests;