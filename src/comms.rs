use serde_derive::*;
use std::net;
use std::marker::PhantomData;
use crate::world;

#[derive(Serialize, Clone, Deserialize, Debug)]
pub enum Action {
	Disconnect,
	Message(String),
	TurnShip(i8),
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TimestampedAction {
	pub timestamp : f64,
	pub action : Action,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub enum Perception {
	ID(usize),
	World(world::World)
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TimestampedPerception {
	pub timestamp : f64,
	pub perception : Perception,
}

#[derive(Clone, Debug)]
pub struct ClientComm {
	pub stream : TypedStream<TimestampedPerception, TimestampedAction>,
	pub timestamp : f64,
	pub online : bool,
}

impl ClientComm {
	pub fn new(tcpstream : net::TcpStream) -> Self {
		ClientComm {
			stream : TypedStream::new(tcpstream),
			timestamp : std::time::UNIX_EPOCH.elapsed().unwrap().as_secs_f64(),
			online : true,
		}
	}

	pub fn authorative_send(&self, perception : Perception) {
		let ts_perc = TimestampedPerception {
			timestamp : self.timestamp,
			perception,
		};
		self.stream.send(&ts_perc);
	}

	pub fn recv(&mut self) -> Result<TimestampedAction, bincode::ErrorKind>  {
		self.stream.recv()
	}

	pub fn disconnect(&mut self) {
		dbg!("There's a disconnect here!");
		self.online = false;
		self.stream.shutdown();
	}

}

#[derive(Debug)]
pub struct TypedStream<S : serde::de::DeserializeOwned + serde::Serialize, R : serde::de::DeserializeOwned + serde::Serialize> {
	pub stream  : net::TcpStream,
	recv_buffer : Vec<u8>,
	marker      : PhantomData<(S, R)>,
}

impl<S : serde::de::DeserializeOwned + serde::Serialize, R : serde::de::DeserializeOwned + serde::Serialize + std::fmt::Debug> TypedStream<S, R> {
	pub fn new(stream : net::TcpStream) -> Self {
		stream.set_nonblocking(true).unwrap();
		Self {
			stream,
			recv_buffer : vec![],
			marker : PhantomData,
		}
	}

	pub fn send(&self, send : &S) {
		bincode::serialize_into(&self.stream, send).unwrap();
	}

	pub fn recv(&mut self) -> Result<R, bincode::ErrorKind> {
		use std::io::Read;
		let mut buff = [10u8; 256];
		loop {
			match self.stream.read(&mut buff) {
				Ok(n) => {
					self.recv_buffer.extend(&buff[..n]);
					if n < 256 { break }
				},
				Err(err) => return Err(bincode::ErrorKind::Io(err)),
			}
		}
		let recv = bincode::deserialize(&self.recv_buffer).map_err(|err| *err)?;
		let size = bincode::serialized_size(&recv).map_err(|err| *err)?;
		self.recv_buffer.drain(..size as usize);
		Ok(recv)
	}

	pub fn shutdown(&mut self) {
		let _ = self.stream.shutdown(net::Shutdown::Both);
	}
}

impl<S : serde::de::DeserializeOwned + serde::Serialize, R : serde::de::DeserializeOwned + serde::Serialize> Clone for TypedStream<S, R> {
	fn clone(&self) -> Self {
		Self {
			stream : self.stream.try_clone().unwrap(),
			recv_buffer : vec![],
			..*self
		}
	}
}