extern crate futures;
extern crate tokio;

use std::sync::Arc;
use std::io::BufReader;

use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

pub mod game;
pub mod room;
pub mod html;

use room::WaitingList;

pub fn process_socket(
    socket: TcpStream,
    waiting: Arc<WaitingList>,
) -> std::io::Result<()> {
    let addr = socket.peer_addr()?;
    println!("Processing new connection {}...", addr);

    socket.set_nodelay(true)?;
    let (reader, writer) = socket.split();
    let reader = BufReader::new(reader);

    let get_name = io::read_until(reader, b'\n', Vec::new())
        .and_then(move |(reader, vec)| {
            if vec.len() == 0 {
                Err(io::Error::from(io::ErrorKind::BrokenPipe))
            } else {
                match String::from_utf8(vec) {
                    Ok(s) => {
                        waiting.insert(addr, s, reader, writer);
                        Ok(())
                    }
                    Err(e) => {
                        Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
                    }
                }
            }
        });

    tokio::spawn(get_name.then(move |result| {
        if let Err(e) = result {
            println!("Connection {} aborted with error: {}", addr, e);
        } else {
            println!("Connection handled: {}", addr);
        }
        Ok(())
    }));

    Ok(())
}
