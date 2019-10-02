# Snake Arena

A simple multiplayer snake game over a network. A little buggy and unpolished,
but nevertheless in a usable state.

## Installation

### Clients

Requires python >= 3.7 (for `asyncio`), and `pyglet` for graphics. Otherwise
should work on all operating systems that `asyncio` and/or `socket` supports.

Once python is installed, from a command prompt or terminal run `pip3 install
pyglet` to install the graphics library. After that everything should just
work, as long as its connecting to a server correctly.

### Server

Requires the [`rust`](https://www.rust-lang.org) language. Install `rustup` as
per the instructions on the `rust` webpage.

Install and run the server using `cargo run` from a command prompt or
terminal. If the server is running on your own computer you may access it at
the IP address `127.0.0.1`, otherwise you will need to find the IP address of
the computer its running on — google how depending on your operating system.
(If it's on the same network it's probably something like `192.168.X.X`.)

## Demo Clients

Currently the following demo clients exist:

- [simple](client/simple.py): A simple, non-networked snake game with no
  actual functionality. Just to get familiar with pyglet.
- [view_once](client/view_once.py): Gets the initial map from the server and
  displays it, with no other functionality.
- [blocking](client/blocking.py): A fully functional (-ish) impmlementation
  using a blocking `socket`s-based implementation.
- [async](client/async.py): A non-blocking implementation using python's
  `asyncio` library that allows for a human to control the snake.
  Still very much bare-bones in terms of usability.
- [view_history](client/view_history.py) Given a URL with the JSON data,
  displays an interactive walkthrough of the round.
- [ai_random](client/ai_random.py) A simple AI that makes a random choice each
  turn.
- [ai_nocollide](client/ai_nocollide.py) A slightly-less-simple AI that'll
  never deliberately kill itself, but is otherwise random.

## Usage

When a client connects to the server it is added to a waiting list. Clients in
the waiting list can be "subscribed" to a room: each room has different
attributes. Clients can only be subscribed to one room at a time.

Once you are happy with the clients subscribed to a room, you can "Start" the
room running. The webpage doesn't automatically update (yet!), so periodically
pressing "Refresh" is required to see the progress of the room. Once the room
is finished it stores the final scores of every player, as well as a record of
the world map at each turn. The room may be played again by pressing "Reset";
note that this also clears the room's history. ("Reset" can also be used to
quit a stuck or long-running room play, if that happens.)

The clients, once the server has started, each receive a message containing
the current state of their connection, as well as (if applicable) an object
describing the current map. Only living clients may respond to the server
(though dead ones can still listen). The only valid responses are `Left`,
`Right` or `Forward`. Note that the connection is newline-delimited, so all
messages sent or received will be terminated by newlines.

## Implementation Notes

The rust code is not particularly well commented, but there should be enough
to follow the main thrust of the work if one is so inclined. Note that most of
the rust code should probably be rewritten in the near future when async-await
stabilises; I started this project before I was aware of the work, and would
probably have avoided many headaches by using them rather than futures.

Of the demo clients, only `async.py`, `view_history` and `ai_nocollide.py` are
particularly well tested... and even then, that just means I'm well aware of
many of their bugs! In particular, `async.py` would dearly love a UX update
and otherwise be made more usable — it is also quite severely buggy in its
current state, though it does perform the base game quite well.

AI-wise, both demo AIs are thoroughly terrible ☺ It wouldn't be all that much
work to write a BFS-based AI to at least improve things slightly, though at
least `nocollide.py` doesn't commit suicide (as often).

(Incidentally, there must be a better way of doing networking combined with
graphics. The awkward method I ended up doing of crossing `asyncio` with
`pyglet` and performing an ugly hack to integrate their event loops was
altogether far too much effort. If anybody knows any better ways of doing it
other than resorting to manually implementing my own `select`-polling on top
of pyglet I'd love to know ☺)
