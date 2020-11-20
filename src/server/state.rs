use std::net;
use std::thread;
use std::sync::mpsc;
use comms::*;

use crate::world;
use crate::comms;
use super::utils;

pub struct Server {
	pub ip              : net::IpAddr,
	pub listener        : net::TcpListener,
	pub world           : world::World,
	pub client_handlers : Vec<thread::JoinHandle<()>>,
	pub clients         : Vec<comms::Client>,
	pub sender          : mpsc::Sender<(usize, comms::TimestampedAction)>,
	pub receiver        : mpsc::Receiver<(usize, comms::TimestampedAction)>,
	pub timestep        : utils::Timer,
	pub authorative_ts  : utils::Timer,
}

impl Server {
	pub fn new() -> Self {
		let (sender, receiver) = mpsc::channel();

		let server_ip = net::IpAddr::V4(net::Ipv4Addr::new(0,0,0,0));

		let socket_addr = net::SocketAddr::new(server_ip, utils::SERVER_PORT);

		let listener = net::TcpListener::bind(socket_addr).unwrap_or_else(|_| panic!("unable to listen on {}", socket_addr));

		Self {
			ip : utils::get_public_ip(),
			world : world::World::new(),
			client_handlers : vec![],
			clients : vec![],
			timestep : utils::Timer::new(),
			authorative_ts : utils::Timer::new(),
			listener,
			sender,
			receiver,
		}
	}

	pub fn handle_client(player_id : usize, client : net::TcpStream, sender : mpsc::Sender<(usize, comms::TimestampedAction)>) {

		const DISCONNECT : TimestampedAction = TimestampedAction {
			timestamp : 0.0,
			action : Action::Disconnect
		};

		let mut incoming_data = bincode::deserialize_from(&client);
		while let Ok(action) = incoming_data {
			sender.send((player_id, action)).expect("Server must have crashed.");
			incoming_data = bincode::deserialize_from(&client);
		}
		if let Err(_) = incoming_data {
			sender.send((player_id, DISCONNECT)).expect("Server already closed.");
		}

		sender.send((player_id, DISCONNECT)).expect("Server already closed.");
		println!("Connection closed with {:?}", client);

	}

	pub fn accept(&mut self, n : usize) {
		println!("Listening on {:?}", net::SocketAddr::new(self.ip, utils::SERVER_PORT));
		for stream in self.listener.incoming() {
			match stream {
				Ok(client) => {

					println!("New connection: {:?}", client);

					let cloned_sender = self.sender.clone();
					let cloned_client = client.try_clone().unwrap();
					let player_id = self.client_handlers.len();
					let join_handle = thread::spawn(move || {
						Self::handle_client(player_id, cloned_client, cloned_sender);
					});

					self.client_handlers.push(join_handle);
					let player_client = comms::Client::new(client);
					player_client.authorative_send_to(Perception::ID(player_id as i32));
					self.clients.push(player_client);
					self.world.ships.push(world::Ship::new());

				},
				Err(e) => {
					panic!("Error: {}", e);
				}
			}
			if self.client_handlers.len() >= n {
				break;
			}
		}
		println!("Stopped listening on {:?}", net::SocketAddr::new(self.ip, utils::SERVER_PORT));
		self.timestep.reset();
	}

	pub fn process(&mut self) -> bool {
		let timestep = self.timestep.reset();
		self.world.update(timestep);

		while let Ok(action) = self.receiver.try_recv() {
			self.clients[action.0].timestamp = action.1.timestamp;

			use Action::*;
			match action.1.action {
				Disconnect => self.clients[action.0].disconnect(),
				_ => self.world.process(action.0, &action.1.action),
			}
		}

		if self.authorative_ts.secs() > 100./1000. {
			self.authorative_ts.reset();
			for client in &self.clients {
				client.authorative_send_to(Perception::World(self.world.clone()));
			}
		}

		if self.clients.iter().any(|c| c.online) { true } else { false }
	}
}

