use std::net;
use std::time;

pub const SERVER_PORT : u16 = 8778;

pub fn get_public_ip() -> net::IpAddr {
	let ip_str = reqwest::blocking::get("https://www.sfml-dev.org/ip-provider.php")
		.expect("Unable to make internet request, are you sure you're connected to the internet?")
		.text()
		.unwrap();

	use std::str::FromStr;
	net::IpAddr::from_str(&ip_str).unwrap()
}

#[derive(Debug)]
pub struct Timer {
	instant : time::Instant,
}

impl Timer {
	pub fn new() -> Self {
		Self {
			instant : time::Instant::now(),
		}
	}

	pub fn secs(&self) -> f32 {
		self.instant.elapsed().as_secs_f32()
	}

	pub fn reset(&mut self) -> f32 {
		let secs = self.secs();
		self.instant = time::Instant::now();
		secs
	}
}