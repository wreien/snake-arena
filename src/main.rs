extern crate tokio;

#[macro_use]
extern crate lazy_static;

use server::game::Tile;
use server::html;
use server::room::{Room, WaitingList};

use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::runtime::Runtime;

#[macro_use]
extern crate warp;
use warp::{http::StatusCode, Filter, Rejection, Reply};

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref ROOMS: Vec<Arc<Mutex<Room>>> = create_rooms();
}

fn manage_rooms(
    waiting_list: Arc<WaitingList>,
) -> impl warp::Filter<Extract = (impl Reply,), Error = Rejection> {
    let with_waitlist = warp::any().map(move || waiting_list.clone());
    use warp::reject::not_found;

    let get_room = |id| {
        ROOMS
            .get(id)
            .cloned()
            .map(|r| (id, r))
            .ok_or_else(not_found)
    };

    let index = warp::path::end()
        .and(with_waitlist.clone())
        .map(|waitlist: Arc<WaitingList>| html::index(&ROOMS, waitlist))
        .map(warp::reply::html);

    let room_page = path!["room" / usize]
        .and(warp::path::end())
        .and(warp::get2())
        .and_then(get_room)
        .untuple_one()
        .and(with_waitlist.clone())
        .and(warp::any().map(|| None))
        .map(html::room_page)
        .map(warp::reply::html);

    let room_request = path!["room" / usize]
        .and(warp::path::end())
        .and(warp::post2())
        .and(warp::body::content_length_limit(1024))
        .and_then(get_room)
        .untuple_one()
        .and(with_waitlist.clone())
        .and(warp::body::form())
        .map(html::room_request)
        .map(warp::reply::html);

    let room_history = path!["room" / usize / "history"]
        .and(warp::path::end())
        .and(warp::get2())
        .and_then(get_room)
        .map(|(_, room): (_, Arc<Mutex<Room>>)| {
            warp::reply::json(&room.lock().unwrap().history)
        });

    let err_404 = warp::any()
        .map(html::page_not_found)
        .map(warp::reply::html)
        .map(|reply| warp::reply::with_status(reply, StatusCode::NOT_FOUND));

    index
        .or(room_page)
        .or(room_request)
        .or(room_history)
        .or(err_404)
}

/// Create a simple room
fn create_simple() -> Arc<Mutex<Room>> {
    use Tile::*;
    Arc::new(Mutex::new(Room::new(
        5, 5, vec![
            Wall,  Wall,  Wall,  Wall,  Wall,
            Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank,
        ], None, 500, "Simple",
        "A very small and simple room for testing with."
    )))
}

/// Create a large room
fn create_large() -> Arc<Mutex<Room>> {
    use Tile::*;
    Arc::new(Mutex::new(Room::new(
        20, 16, vec![
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Wall,  Wall,  Wall,  Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Wall,  Blank, Wall,  Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Wall,  Wall,  Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Wall,  Wall,  Blank, Blank, Blank,
            Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Wall,  Wall,  Wall,  Wall,  Wall,  Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Wall,  Wall,  Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Wall,  Wall,  Blank, Blank, Blank, Blank, Blank,
            Wall,  Wall,  Wall,  Blank, Wall,  Wall,  Wall,  Wall,  Blank, Blank, Blank, Blank, Wall,  Wall,  Wall,  Wall,  Blank, Wall,  Wall,  Wall,
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank,
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Wall,  Blank, Blank,
            Wall,  Wall,  Wall,  Wall,  Wall,  Wall,  Wall,  Wall,  Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Wall,  Wall,  Wall,
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank, Blank, Blank, Blank, Blank,
        ], None, 12_000, "Large",
        "A very large room with interesting wall placing."
    )))
}

#[rustfmt::skip]
fn create_rooms() -> Vec<Arc<Mutex<Room>>> {
    use Tile::*;
    let boxed = Arc::new(Mutex::new(Room::new(
        10, 10, vec![
            Wall, Wall,  Wall,  Wall,  Wall,  Wall,  Wall,  Wall,  Wall,  Wall,
            Wall, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,
            Wall, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,
            Wall, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,
            Wall, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,
            Wall, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,
            Wall, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,
            Wall, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,
            Wall, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank, Wall,
            Wall, Wall,  Wall,  Wall,  Wall,  Wall,  Wall,  Wall,  Wall,  Wall,
        ], None, 1_000, "Boxed",
        "A moderate-sized room that is boxed in around the outside."
    )));

    let speckled = Arc::new(Mutex::new(Room::new(
        8, 8, vec![
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Wall,  Blank, Blank, Blank,
            Blank, Wall,  Wall,  Blank, Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank,
            Blank, Blank, Wall,  Blank, Blank, Wall,  Wall,  Blank,
            Blank, Blank, Blank, Blank, Blank, Blank, Wall,  Blank,
            Blank, Wall,  Blank, Wall,  Blank, Blank, Blank, Blank,
            Blank, Blank, Blank, Blank, Blank, Blank, Blank, Blank,
        ], None, 4_000, "Speckled",
        "A medium-sized room with random walls placed in the centre."
    )));

    vec![
        create_simple(),
        create_simple(),
        create_simple(),
        create_simple(),
        create_simple(),
        create_simple(),
        boxed,
        speckled,
        create_large(),
        create_large(),
        create_large(),
        create_large(),
        create_large(),
        create_large(),
    ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Preparing rooms...");
    lazy_static::initialize(&ROOMS);
    let waiting_list = Arc::new(WaitingList::new());

    let serve_waitlist = waiting_list.clone();
    let s_addr = "0.0.0.0:3001".parse()?;
    let socket = TcpListener::bind(&s_addr)?;
    println!("Execution server listening on {}", s_addr);
    let tcp_srv = socket
        .incoming()
        .for_each(move |socket| server::process_socket(socket, serve_waitlist.clone()))
        .map_err(|e| eprintln!("Error occurred: {:?}", e));

    let w_addr = "0.0.0.0:80".parse::<SocketAddr>()?;
    let warp_srv = warp::serve(manage_rooms(waiting_list)).bind(w_addr);
    println!("HTTP server listening on {}", w_addr);

    let mut rt = Runtime::new()?;
    rt.spawn(tcp_srv);
    rt.spawn(warp_srv);
    rt.shutdown_on_idle().wait().unwrap();

    Ok(())
}
