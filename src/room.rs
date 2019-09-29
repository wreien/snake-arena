//! A game room.

use std::collections::HashMap;
use std::io::{BufReader, Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::{mpsc, oneshot};

use futures::future::Either;

use crate::game::{Map, SnakeID, Tile};

/// Possible requests we can get from the clients
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum Request {
    /// Turn their snake left
    Left,

    /// Turn their snake right
    Right,

    /// Let their snake go forwards
    Forward,
}

type Reader = BufReader<io::ReadHalf<TcpStream>>;
type Writer = io::WriteHalf<TcpStream>;
type NamedSocket = (String, Reader, Writer);

/// People that are waiting for a room
#[derive(Debug, Default)]
pub struct WaitingList(Mutex<HashMap<SocketAddr, NamedSocket>>);

impl WaitingList {
    /// Create the waiting list
    pub fn new() -> Self {
        WaitingList(Mutex::new(HashMap::new()))
    }

    /// Insert the socket into the list.
    ///
    /// Returns `true` if it overwrote an existing waiter.
    pub fn insert(
        &self,
        addr: SocketAddr,
        name: String,
        reader: Reader,
        writer: Writer,
    ) -> bool {
        self.0
            .lock()
            .unwrap()
            .insert(addr, (name, reader, writer))
            .is_some()
    }

    /// Moves the waiter to the given room.
    pub fn subscribe(&self, addr: &SocketAddr, room: &mut Room) -> std::io::Result<()> {
        let mut data = self.0.lock().unwrap();
        if let Some(waiter) = data.remove(addr) {
            if let RoomState::Waiting = room.state {
                room.players.insert(*addr, waiter);
                Ok(())
            } else {
                data.insert(*addr, waiter);
                Err(Error::new(
                    ErrorKind::InvalidInput,
                    "provided room is already in progress",
                ))
            }
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "address not in wait queue",
            ))
        }
    }

    /// Removes a socket from the waiting list.
    ///
    /// Returns `true` if it removed something.
    pub fn remove(&self, addr: &SocketAddr) -> bool {
        self.0.lock().unwrap().remove(addr).is_some()
    }

    /// Clear the waiting list.
    pub fn clear(&self) {
        self.0.lock().unwrap().clear();
    }

    /// Get the list of people in the waiting list
    pub fn waiters(&self) -> Vec<(SocketAddr, String)> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|(&addr, (name, _, _))| (addr, name.clone()))
            .collect()
    }
}

#[derive(Debug)]
enum RoomState {
    Waiting,
    Playing {
        map: Arc<Mutex<Map>>,
        addrs: HashMap<SocketAddr, (String, SnakeID)>,
        breaker: oneshot::Sender<()>,
    },
    Finished {
        scores: HashMap<SocketAddr, (String, usize)>,
    },
}

#[derive(Debug)]
pub enum State {
    Waiting {
        players: Vec<(SocketAddr, String)>,
    },
    Playing {
        map: Arc<Mutex<Map>>,
        players: HashMap<SocketAddr, (String, SnakeID)>,
    },
    Finished {
        scores: HashMap<SocketAddr, (String, usize)>,
    },
}

/// The room that snakes play in
#[derive(Debug)]
pub struct Room {
    state: RoomState,
    players: HashMap<SocketAddr, NamedSocket>,

    pub history: Vec<Map>,

    /// How long between each snake movement.
    /// `None` means it just goes as soon as it receives all results.
    pub timestep: Option<Duration>,

    /// Map width
    pub width: usize,

    /// Map height
    pub height: usize,

    /// Initial tile state; this should just be `Tile::Blank` and `Tile::Wall`.
    pub tiles: Vec<Tile>,

    /// The name of the room.
    pub name: String,

    /// The description for the room.
    pub description: String,
}

impl Room {
    /// Create a room with the given initial map state.
    pub fn new<S1: Into<String>, S2: Into<String>>(
        width: usize,
        height: usize,
        tiles: Vec<Tile>,
        timestep: Option<Duration>,
        name: S1,
        description: S2,
    ) -> Self {
        Room {
            state: RoomState::Waiting,
            players: HashMap::new(),
            history: Vec::new(),
            timestep,
            width,
            height,
            tiles,
            name: name.into(),
            description: description.into(),
        }
    }

    /// Remove a socket from the waiting list.
    pub fn unsubscribe(
        &mut self,
        addr: &SocketAddr,
        list: &WaitingList,
    ) -> std::io::Result<()> {
        self.players
            .remove(addr)
            .map(|(name, reader, writer)| list.insert(*addr, name, reader, writer))
            .map(|_| ())
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "address not in room"))
    }

    /// Reset the room to its initial state.
    ///
    /// This removes all players and subscribers, resets the map, and goes back to the
    /// `Waiting` state.
    pub fn reset(&mut self) -> Result<(), &'static str> {
        self.players.clear();
        let old_state = std::mem::replace(&mut self.state, RoomState::Waiting);

        match old_state {
            RoomState::Playing { breaker, .. } => {
                breaker.send(()).map_err(|_| "failed to send reset signal")
            }
            _ => Ok(()),
        }
    }

    /// Return the current room state.
    pub fn get_state(&self) -> State {
        match &self.state {
            RoomState::Waiting => State::Waiting {
                players: self.players.iter()
                    .map(|(&addr, (name, _, _))| (addr, name.clone()))
                    .collect(),
            },
            RoomState::Playing { map, addrs, .. } => State::Playing {
                map: map.clone(),
                players: addrs.clone(),
            },
            RoomState::Finished { scores } => State::Finished {
                scores: scores.clone(),
            },
        }
    }
}

/// Helper to turn errors into `std::io::ErrorKind::BrokenPipe`
fn to_broken_pipe<E: ToString>(e: E) -> Error {
    Error::new(ErrorKind::BrokenPipe, e.to_string())
}

/// Set up the client for game execution.
///
/// Returns a sink/stream pair for communicating with the client.
fn setup_client(
    id: usize,
    addr: SocketAddr,
    reader: Reader,
    writer: Writer,
) -> (
    impl Sink<SinkItem = String, SinkError = Error> + Send,
    impl Stream<Item = Request, Error = Error> + Send,
) {
    let (tx_to_sock, rx_from_map) = mpsc::unbounded_channel::<String>();
    let (tx_to_map, rx_from_sock) = mpsc::unbounded_channel::<Request>();

    let tx_to_sock = tx_to_sock.sink_map_err(to_broken_pipe);
    let tx_to_map = tx_to_map.sink_map_err(to_broken_pipe);
    let rx_from_sock = rx_from_sock.map_err(to_broken_pipe);
    let rx_from_map = rx_from_map.map_err(to_broken_pipe);

    let requests = io::lines(BufReader::new(reader))
        .and_then(move |line: String| {
            println!("{} ({}) received: {}", addr, id, line);
            match line.as_str() {
                "Forward" => Ok(Request::Forward),
                "Left" => Ok(Request::Left),
                "Right" => Ok(Request::Right),
                _ => Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("couldn't parse line: {}", line),
                )),
            }
        })
        .forward(tx_to_map)
        .map(|_| ());

    let responses =
        io::write_all(writer, format!("{{\"state\":\"start\",\"id\":{}}}\n", id))
            .map(move |(writer, _)| writer)
            .and_then(move |writer| {
                rx_from_map.fold(writer, |writer, msg| {
                    io::write_all(writer, format!("{}\n", msg)).map(|(writer, _)| writer)
                })
            });

    let connection = requests.select2(responses).then(
        move |result| -> Box<dyn Future<Item = _, Error = _> + Send> {
            match result {
                // things are OK; just send the other half
                Ok(Either::A((_, responses))) => Box::new(responses.map(|_| ())),
                // we've finished sending responses; don't wait for more requests!
                Ok(Either::B(_)) => Box::new(future::ok(())),
                // bad request; notify client and close connection
                Err(Either::A((e, responses))) => {
                    Box::new(responses.and_then(move |writer| {
                        io::write_all(
                            writer,
                            format!("{{\"state\":\"error\",\"msg\":{}}}\n", e),
                        )
                        .and_then(|_| future::err(e))
                    }))
                }
                // couldn't respond: just die, not much else to do
                Err(Either::B((e, _))) => Box::new(future::err(e)),
            }
        },
    );

    tokio::spawn(connection.then(move |result| {
        if let Err(e) = result {
            println!("Connection {} closed with error: {}", addr, e);
        } else {
            println!("Connection closed: {}", addr);
        }
        Ok(())
    }));

    (tx_to_sock, rx_from_sock)
}

/// Do one step of client interaction.
fn do_client_step<'a, T, R>(
    id: SnakeID,
    tx: T,
    rx: R,
    map: Arc<Mutex<Map>>,
    map_json: String,
    timestep: Option<Duration>,
) -> Box<dyn Future<Item = (SnakeID, T, R), Error = std::io::Error> + 'a + Send>
where
    T: Sink<SinkItem = String, SinkError = std::io::Error> + 'a + Send,
    R: Stream<Item = Request, Error = std::io::Error> + 'a + Send,
{
    // don't bother receiving anything if they're dead
    if !map.lock().unwrap().is_alive(id) {
        let json = format!("{{\"state\":\"dead\",\"map\":{}}}", map_json);
        return Box::new(tx.send(json).map(move |tx| (id, tx, rx)));
    }

    let rmap = map.clone();
    let json = format!("{{\"state\":\"playing\",\"map\":{}}}", map_json);
    let action = tx.send(json).and_then(move |tx| {
        rx.into_future()
            .map_err(|(e, _)| e)
            .and_then(move |(req, rx)| {
                match req {
                    Some(Request::Forward) => {}
                    Some(Request::Left) => rmap.lock().unwrap().turn_left(id),
                    Some(Request::Right) => rmap.lock().unwrap().turn_right(id),
                    None => return Err(to_broken_pipe("no request received")),
                }
                Ok((id, tx, rx))
            })
    });

    let action = action.map_err(move |e| {
        // on error, remove the associated snake from the map
        map.lock().unwrap().delete_snake(id);
        e
    });

    if let Some(duration) = timestep {
        let action = action
            .timeout(duration)
            .map_err(|e| Error::new(ErrorKind::TimedOut, e.to_string()));
        Box::new(action)
    } else {
        Box::new(action)
    }
}

/// Execute the server work once we have all our client work done
fn do_server_step<T>(
    room: Arc<Mutex<Room>>,
    map: Arc<Mutex<Map>>,
    socket_txs: T,
) -> Result<future::Loop<T, (Arc<Mutex<Room>>, T)>, ()> {
    // always lock room before map
    let mut room_inner = room.lock().unwrap();
    let mut map_inner = map.lock().unwrap();
    match map_inner.clone().step() {
        Ok(map) => {
            let map = std::mem::replace(&mut *map_inner, map);
            room_inner.history.push(map);
            drop(room_inner);
            Ok(future::Loop::Continue((room, socket_txs)))
        }
        Err(scores) => {
            room_inner.history.push(map_inner.clone());
            if let RoomState::Playing { addrs, .. } = &room_inner.state {
                let scores = scores
                    .into_iter()
                    .map(|(id, scr)| {
                        (
                            addrs
                                .iter()
                                .find(|&(_, &(_, old_id))| old_id == id)
                                .map(|(&addr, (name, _))| (addr, name.clone()))
                                .expect("addr -> snake table incomplete"),
                            scr,
                        )
                    })
                    .map(|((addr, name), scr)| (addr, (name, scr)))
                    .collect();
                room_inner.state = RoomState::Finished { scores };
                Ok(future::Loop::Break(socket_txs))
            } else {
                println!("room in weird state?");
                Err(())
            }
        }
    }
}

/// Shut things off and start playing
///
/// Returns `false` if the room failed to start.
pub fn run(room: Arc<Mutex<Room>>) -> bool {
    let mut room_inner = room.lock().unwrap();

    // make sure the room is in a good state
    let good = match &room_inner.state {
        RoomState::Waiting => !room_inner.players.is_empty(),
        _ => false,
    };
    if !good {
        return false;
    }

    // let the players know we've started by providing them their ID
    // this also clears the player list
    let (addrs, sockets): (HashMap<_, _>, Vec<_>) = room_inner
        .players
        .drain()
        .enumerate()
        .map(|(id, (addr, (name, reader, writer)))| {
            let (tx, rx) = setup_client(id, addr, reader, writer);
            ((addr, (name, id)), (id, tx, rx))
        })
        .unzip();

    // update the room state; we can drop the lock when we're done here
    let map = Arc::new(Mutex::new(Map::new(
        room_inner.width,
        room_inner.height,
        room_inner.tiles.clone(),
        addrs.iter().map(|(_, &(_, id))| id).collect(),
    )));
    let (breaker_send, breaker_recv) = oneshot::channel();
    room_inner.state = RoomState::Playing {
        map,
        addrs,
        breaker: breaker_send,
    };
    drop(room_inner);

    let task = future::loop_fn((room, sockets), move |(room, sockets)| {
        let room_inner = room.lock().unwrap();
        if let RoomState::Playing { map, .. } = &room_inner.state {
            let map = map.clone();
            let timestep = room_inner.timestep;
            drop(room_inner); // unlock the mutex now we have the map

            // our serialize function will never fail
            let map_inner = map.lock().unwrap();
            let json = serde_json::to_string(&*map_inner).unwrap();
            drop(map_inner); // unlock the mutex now we have the representation

            let futs = sockets.into_iter().map(|(id, tx, rx)| {
                do_client_step(id, tx, rx, map.clone(), json.clone(), timestep)
            });

            stream::futures_unordered(futs)
                .map(Some)
                .or_else(|err| {
                    // deal with errors by just ditching the socket
                    println!("Error: {}", err);
                    future::ok::<_, ()>(None)
                })
                .filter_map(|x| x)
                .collect()
                .and_then(move |sockets| do_server_step(room, map, sockets))
        } else {
            panic!("Error: room in weird state?");
        }
    });

    // notify clients that the game is over
    let task = task.and_then(|sockets| {
        // send a "done" message to all sockets, ignoring errors
        let futs = sockets.into_iter().map(|(_, tx, _)| {
            tx.send("{\"state\":\"done\"}".into())
                .and_then(|mut tx| tx.close())
                .then(|_| Ok(()))
        });
        future::join_all(futs).map(|_| ())
    });

    // cancel task if we get a message from the oneshot
    let task = task.select(breaker_recv.map_err(|_| ()));

    tokio::spawn(task.then(|_| {
        println!("Room running task finished.");
        Ok(())
    }));

    true
}
