//! Generate human-usable webpages and API endpoints to interact with the game.

extern crate markup;

use crate::game::SnakeID;
use crate::room::{self, Room, State, WaitingList};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

markup::define! {
    Page(contents: Vec<Box<dyn markup::Render>>, alert: Option<(String, String)>) {
        {markup::doctype()}
        html[lang = "en"] {
            head {
                meta[charset = "utf-8"];
                meta[
                    name = "viewport",
                    content = "width=device-width, initial-scale=1, shrink-to-fit=no"
                ];
                link[
                    rel = "stylesheet",
                    href = "https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css",
                    integrity = "sha384-ggOyR0iXCbMQv3Xipma34MD+dH/1fQ784/j6cY/iJTQUOhcWr7x9JvoRxT2MZw1T",
                    crossorigin = "anonymous",
                ];
                title { "Snake Arena" }
                style { {markup::raw("nav {margin-bottom: 1.5rem;}")} }
            }
            body {
                nav.navbar."navbar-expand-lg"."navbar-dark"."bg-dark" {
                    div.container {
                        a."navbar-brand" [href = "/"] { "Snake Arena" }
                    }
                }

                main.container {
                    @if let Some((class, text)) = &*(alert) {
                        div.alert."alert-dismissable".fade.show
                            .{format!("alert-{}", class)} [role = "alert"]
                        {
                            {text}
                            button.close [
                                type = "button",
                                "data-dismiss" = "alert",
                                "aria-label" = "Close",
                            ] {
                                span ["aria-hidden" = "true"] {
                                    {markup::raw("&times;")}
                                }
                            }
                        }
                    }

                    @for c in contents.iter() {
                        {c.as_ref()}
                    }
                }

                // bootstrap script
                script[
                    src = "https://code.jquery.com/jquery-3.3.1.slim.min.js",
                    crossorigin = "anonymous",
                ] {}
                script[
                    src = "https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.14.7/umd/popper.min.js",
                    crossorigin = "anonymous",
                ] {}
                script[
                    src = "https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/js/bootstrap.min.js",
                    crossorigin = "anonymous",
                ] {}
            }
        }
    }

    Index(rooms: Vec<(String, String, String, usize)>, waiters: Vec<String>) {
        h1 { "Snake Arena: Control Panel" }
        hr;
        h3 { "Available Rooms" }
        table.table {
            thead."thead-light" {
                tr {
                    th[scope = "col"] { "ID" }
                    th[scope = "col"] { "Name" }
                    th[scope = "col"] { "Description" }
                    th[scope = "col"] { "State" }
                    th[scope = "col"] { "#Players" }
                }
            }
            tbody {
                @for (i, (n, d, s, p)) in rooms.iter().enumerate() {
                    tr {
                        th[scope = "row"] { {i} }
                        td { a[href = format!("/room/{}/", i)] { {n} } }
                        td { {d} }
                        td { {s} }
                        td { {p} }
                    }
                }
            }
        }
        h3 { "Waiters" }
        p { "Navigate to a room's page if you want to subscribe a connection
             to the room." }
        ul {
            @for w in waiters.iter() {
                li { {w} }
            }
        }
    }

    RoomHeader(id: usize, name: String, desc: String) {
        h1 { "Room #" {id} " — " {name} }
        p.lead { {desc} }
    }

    RoomWaiting(players: Vec<(String, String)>) {
        p { b { "Room status:" } " waiting to begin." }
        {RoomControlButtons { include_start: !players.is_empty() }}
        hr;
        h3 { "In queue" }
        @if players.is_empty() {
            p { "There are no players in the room." }
        } else {
            table.table {
                thead."thead-light" {
                    tr {
                        th[scope = "col"] { "ID" }
                        th[scope = "col"] { "Address" }
                        th."text-right"[scope = "col"] { "Actions" }
                    }
                }
                tbody {
                    @for (i, (a, n)) in players.iter().enumerate() {
                        tr {
                            td."align-middle" { {i} }
                            td."align-middle" { {format!("{} — {}", a, n)} }
                            td."align-middle"."text-right" {
                                form."inline-form" [ method = "post" ] {
                                    input[hidden? = true, name = "waiter", value = {a}];
                                    button.btn."btn-outline-danger"."btn-sm" [
                                        type = "submit",
                                        name = "unsubscribe",
                                    ] {
                                        "Unsubscribe"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    RoomPlaying(scores: Vec<(SnakeID, String, usize)>) {
        p { b { "Room status:" } " in progress." }
        {RoomControlButtons { include_start: false }}
        hr;
        h3 { "Current scores" }
        table.table {
            thead."thead-light" {
                tr {
                    th[scope = "col"] { "ID" }
                    th[scope = "col"] { "Address" }
                    th[scope = "col"] { "Score" }
                }
            }
            tbody {
                @for (i, a, s) in scores.iter() {
                    tr {
                        th[scope = "row"] { {i} }
                        td { {a} }
                        td { {s} }
                    }
                }
            }
        }
    }

    RoomFinished(scores: Vec<(String, usize)>) {
        p { b { "Room status:" } " finished." }
        {RoomControlButtons { include_start: false }}
        hr;
        h3 { "Final scores" }
        table.table {
            thead."thead-light" {
                tr {
                    th[scope = "col"] { "Address" }
                    th[scope = "col"] { "Score" }
                }
            }
            tbody {
                @for (a, s) in scores.iter() {
                    tr {
                        td { {a} }
                        td { {s} }
                    }
                }
            }
        }
    }

    RoomControlButtons(include_start: bool) {
        a.btn."mb-2"."btn-outline-info"[href="./history"] { "Get room history (JSON)" }
        form[method = "post"] {
            button.btn."mr-2".{
                if *include_start { "btn-primary" } else { "btn-secondary" }
            } [
                type = "submit",
                name = "start_room",
                disabled? = !*include_start,
            ] { "Start " }
            button.btn."btn-success"."mr-2"[
                onclick = "window.location.href=window.location.href;"
            ] { "Refresh" }
            button.btn."btn-danger"."mr-2"[
                type = "submit",
                name = "reset_room",
            ] { "Reset " }
        }
    }

    WaitDropdown(waiters: Vec<(String, String)>) {
        h3 { "Waiters" }
        @if waiters.is_empty() {
            p { "There are no connections waiting for a room." }
        } else {
            form[method = "post"] {
                div."form-group" {
                    @for (i, (addr, name)) in waiters.iter().enumerate() {
                        div."form-check" {
                            input."form-check-input"[
                                type = "radio",
                                name = "waiter",
                                id = {format!("waiters-{}", i)},
                                value = {addr},
                                required? = true,
                            ];
                            label."form-check-label"[for = {format!("waiters-{}", i)}] {
                                {format!("{} — {}", addr, name)}
                            }
                        }
                    }
                }
                button.btn."btn-primary"."mr-2"[
                    type = "submit",
                    name = "subscribe",
                ] { "Subscribe" }
                button.btn."btn-outline-secondary"."mr-2"[
                    type = "submit",
                    name = "kill",
                ] { "Kill" }
                button.btn."btn-danger"."mr-2"[
                    type = "submit",
                    name = "kill_all",
                    onclick = "$('input').prop('required', false)",
                ] { "Kill all" }
            }
        }
    }

    NotFound() {
        p { "This is not the page you were looking for." }
    }
}

pub fn index(rooms: &[Arc<Mutex<Room>>], waiting_list: Arc<WaitingList>) -> String {
    let rooms: Vec<_> = rooms
        .iter()
        .cloned()
        .map(|room| {
            let room_inner = room.lock().unwrap();
            let (state, members) = match room_inner.get_state() {
                State::Waiting { players } => ("Waiting", players.len()),
                State::Playing { players, .. } => ("Playing", players.len()),
                State::Finished { scores } => ("Finished", scores.len()),
            };
            (
                room_inner.name.clone(),
                room_inner.description.clone(),
                state.to_owned(),
                members,
            )
        })
        .collect();

    let waiters: Vec<_> = waiting_list
        .waiters()
        .iter()
        .map(|(addr, name)| format!("{} — {}", addr, name))
        .collect();

    let index = Box::new(Index { rooms, waiters });
    Page {
        contents: vec![index],
        alert: None,
    }
    .to_string()
}

pub fn room_page(
    id: usize,
    room: Arc<Mutex<Room>>,
    waiting_list: Arc<WaitingList>,
    alert: Option<(String, String)>,
) -> String {
    let mut contents: Vec<Box<dyn markup::Render>> = Vec::new();

    let room_inner = room.lock().unwrap();
    contents.push(Box::new(RoomHeader {
        id,
        name: room_inner.name.clone(),
        desc: room_inner.description.clone(),
    }));

    match room_inner.get_state() {
        State::Waiting { players } => contents.push(Box::new(RoomWaiting {
            players: players
                .iter()
                .map(|(addr, name)| (addr.to_string(), name.clone()))
                .collect(),
        })),
        State::Playing { map, players } => {
            let map = map.lock().unwrap();
            let mut scores: Vec<_> = players
                .iter()
                .map(|(&addr, (name, id))| (*id, format!("{} — {}", addr, name)))
                .map(|(id, addr)| (id, addr, *map.scores.get(&id).unwrap_or(&0)))
                .collect();
            scores.sort_unstable_by_key(|&(id, _, _)| id);
            contents.push(Box::new(RoomPlaying { scores }));
        }
        State::Finished { scores } => {
            contents.push(Box::new(RoomFinished {
                scores: scores
                    .iter()
                    .map(|(a, (n, s))| (format!("{} — {}", a, n), *s))
                    .collect(),
            }));
        }
    }

    let waiters = waiting_list
        .waiters()
        .into_iter()
        .map(|(addr, name)| (addr.to_string(), name))
        .collect();
    contents.push(Box::new(WaitDropdown { waiters }));
    Page { contents, alert }.to_string()
}

#[allow(clippy::implicit_hasher)]
pub fn room_request(
    id: usize,
    room: Arc<Mutex<Room>>,
    waiting: Arc<WaitingList>,
    form: HashMap<String, String>,
) -> String {
    fn fix<E: ToString>(e: E) -> String {
        e.to_string()
    }

    fn to_alert_success<T: ToString>(t: T) -> Option<(String, String)> {
        Some(("success".to_owned(), t.to_string()))
    }
    fn to_alert_error<T: ToString>(t: T) -> Option<(String, String)> {
        Some(("danger".to_owned(), t.to_string()))
    }
    fn to_alert<T: ToString, U: ToString>(r: Result<T, U>) -> Option<(String, String)> {
        match r {
            Ok(msg) => to_alert_success(msg),
            Err(msg) => to_alert_error(msg),
        }
    }

    let alert = if form.contains_key("subscribe") {
        let room_inner = &mut room.lock().unwrap();
        to_alert(
            form.get("waiter")
                .ok_or_else(|| "missing field: waiter".to_owned())
                .and_then(|addr| addr.parse::<SocketAddr>().map_err(fix))
                .and_then(|addr| waiting.subscribe(&addr, room_inner).map_err(fix))
                .map(|_| "Subscribed connection to room."),
        )
    } else if form.contains_key("unsubscribe") {
        let room_inner = &mut room.lock().unwrap();
        to_alert(
            form.get("waiter")
                .ok_or_else(|| "missing field: waiter".to_owned())
                .and_then(|addr| addr.parse::<SocketAddr>().map_err(fix))
                .and_then(|addr| room_inner.unsubscribe(&addr, &waiting).map_err(fix))
                .map(|_| "Removed connection from room."),
        )
    } else if form.contains_key("kill") {
        to_alert(
            form.get("waiter")
                .ok_or_else(|| "missing field: waiter".to_owned())
                .and_then(|addr| addr.parse::<SocketAddr>().map_err(fix))
                .map(|addr| {
                    if waiting.remove(&addr) {
                        "Successfully killed the connection."
                    } else {
                        "Nothing to do."
                    }
                }),
        )
    } else if form.contains_key("kill_all") {
        waiting.clear();
        to_alert_success("Success!")
    } else if form.contains_key("start_room") {
        if room::run(room.clone()) {
            to_alert_success("Started room execution.")
        } else {
            to_alert_error("Room failed to start.")
        }
    } else if form.contains_key("reset_room") {
        let room_inner = &mut room.lock().unwrap();
        to_alert(room_inner.reset().map(|_| "Room reset successfully."))
    } else {
        None
    };

    room_page(id, room, waiting, alert)
}

pub fn page_not_found() -> String {
    Page {
        contents: vec![Box::new(NotFound {})],
        alert: None,
    }
    .to_string()
}
