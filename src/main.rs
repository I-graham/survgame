use std::env;

mod common;
mod server;
mod client;

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
	}

}

#[cfg(test)]
mod tests;