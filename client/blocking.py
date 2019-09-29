#!/usr/bin/env python3

import pyglet
from pyglet.window import key

import socket
import json
import sys
import itertools

from snake import *


# create a connection to the server;
# we'll wrap it in a class to make some things easier

class Socket:
    """A TCP socket that reads a whole line at a time."""

    def __init__(self, host: str, port: int):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((host, port))
        self.sock.setsockopt(socket.SOL_TCP, socket.TCP_NODELAY, 1)
        self.sock.sendall(b'blocking.py\n')
        self.remainder = b""  # leftover line bits

    def readline(self) -> bytes:
        """Get one line of data from the server."""
        while True:
            # if we have a line, return it
            nlpos = self.remainder.find(b"\n")
            if nlpos != -1:
                line, self.remainder = self.remainder.split(b"\n", 1)
                return line

            # otherwise, get a new chunk from the server and try again
            data = self.sock.recv(1024)
            if not data:
                print("Connection killed!")
                sys.exit(1)
            self.remainder = self.remainder + data

    def readjson(self) -> dict:
        """Same as readline, but returns a JSON object."""
        return json.loads(self.readline())

    def write(self, data: bytes):
        """Sends the given string to the server."""
        data = data + b'\n'
        sent = self.sock.send(data)
        while sent < len(data):
            sent = sent + self.sock.send(data[sent:])


sock = Socket("127.0.0.1", 2999)


# get things started

print("Waiting for game start...")
data = sock.readjson()
my_id = data["id"]
print("Starting!")


# create the map

class World:
    """
    Keeps track of all of the snakes and world grid.

    sock: The socket used to connect to the server
    id: Which snake we are
    map: The tilegrid of walls, snakes, and doodahs
    map_size: The dimensions of the map as (width, height)
    tile_size: How large a tile to draw (in pixels)
    """
    def __init__(self, sock: Socket, tile_size: int, my_id: int):
        self.images = {
            "Blank": load_tile("eye_grid.png", tile_size),
            "Wall": load_tile("v.png", tile_size),
            "Doodah": load_tile("tree.png", tile_size),
            "SnakeHead": load_tile("me.png", tile_size),
            "SnakeBody": load_tile("me.png", tile_size),
        }
        self.sock = sock
        self.id = my_id
        self.map = []
        self.map_size = (0, 0)
        self.tile_size = tile_size
        # now we're set up, we need to get the current map from the server
        self.get_world_state()

    def get_world_state(self):
        """Take an update from the server and rebuild the map from it."""
        data = self.sock.readjson()
        if data["state"] != "playing":
            print("Problem: state =", data["state"])
            print(data)
            sys.exit(1)
        self.map_size = (data["map"]["width"], data["map"]["height"])
        tiles = (self.images[x["type"]] for x in data["map"]["tiles"])
        positions = (
            Position(x, y, self.map_size)
            for y in range(self.map_size[1])
            for x in range(self.map_size[0])
        )
        self.map = [Sprite(pos, img) for pos, img in zip(positions, tiles)]

    def go(self, direction: str):
        """
        Given a direction 'Left', 'Right', or 'Forward', send it to the server.
        When done update the world state.
        """
        sock.write(direction.encode('utf-8'))
        self.get_world_state()

    def draw(self):
        """Draw the map."""
        for sprite in self.map:
            sprite.draw()


world = World(sock=sock, tile_size=64, my_id=my_id)


# Set up the window; we'll just draw for now

window = pyglet.window.Window(
    width=world.map_size[0] * world.tile_size,
    height=world.map_size[1] * world.tile_size
)
fps_display = pyglet.window.FPSDisplay(window=window)

@window.event
def on_draw():
    window.clear()
    world.draw()
    fps_display.draw()


# we add in a handler to move our snake from the keyboard

@window.event
def on_key_press(symbol, modifiers):
    if symbol == key.LEFT:
        world.go("Left")
    elif symbol == key.RIGHT:
        world.go("Right")
    elif symbol == key.UP:
        world.go("Forward")


# Set up the event loop and run the game

pyglet.app.run()
