# Snake Arena

## Installation

### Clients

Requires python >= 3.7 (for `asyncio`), and `pyglet` for graphics. Otherwise
should work on all operating systems that `asyncio` and/or `socket` supports.

### Server

Requires [`rust`](https://www.rust-lang.org). Install and run the server using
`cargo run`.

## Demo Clients

Currently the following demo clients exist:

- [simple](client/simple.py): A simple, non-networked snake game with no
  actual functionality. Just to get familiar with pyglet.
- [view_once](client/view_once.py): Gets the initial map from the server and
  displays it, with no other functionality.
- [blocking](client/blocking.py): A fully functional (-ish) impmlementation
  using a blocking `socket`s-based implementation.
- [async](client/async.py): A non-blocking implementation using python's
  `asyncio` library.

