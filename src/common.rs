#[allow(dead_code)]

use std::net;

pub const SERVER_PORT : u16 = 8778;

pub fn get_public_ip() -> net::IpAddr {
	let ip_str = reqwest::blocking::get("https://www.sfml-dev.org/ip-provider.php")
		.expect("Unable to make internet request, are you sure you're connected to the internet?")
		.text()
		.unwrap();

	use std::str::FromStr;
	net::IpAddr::from_str(&ip_str).unwrap()
}