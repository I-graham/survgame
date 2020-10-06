use super::*;

#[test]
fn public_ip() {
	let myip = common::get_public_ip();
	println!("{:?}", myip);
}