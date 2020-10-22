use super::*;

#[test]
fn public_ip() {
	let myip = utils::get_public_ip();
	println!("{:?}", myip);
}