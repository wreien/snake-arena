#!/usr/bin/env python3

import pyglet
import json
import socket
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
        self.sock.sendall(b'view_once.py\n')
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


sock = Socket("127.0.0.1", 2999)


# get things started

print("Waiting for game start...")
data = sock.readjson()
MY_ID = data["id"]

data = sock.readjson()
# we only care about the map at the moment, so...
data = data["map"]


# define some constants

TILE_SIZE = 64
MAP_SIZE = (data["width"], data["height"])


# create the map

images = {
    "Blank": load_tile("empty.png", TILE_SIZE),
    "Wall": load_tile("wall.png", TILE_SIZE),
    "Doodah": load_tile("star.png", TILE_SIZE),
    "SnakeHead": load_tile("yellow_ring.png", TILE_SIZE),
    "SnakeBody": load_tile("yellow_circle.png", TILE_SIZE),
}

tiles = (images[x["type"]] for x in data["tiles"])
positions = (  # this is backwards from my intuition, but whatever :)
    Position(x, y, MAP_SIZE)
    for y in range(data["height"])
    for x in range(data["width"])
)
curr_map = [Sprite(pos, img) for pos, img in zip(positions, tiles)]

# make sure everything is fine and dandy
assert len(curr_map) == MAP_SIZE[0] * MAP_SIZE[1]


# Set up the window; we'll just draw for now

window = pyglet.window.Window(
    width=MAP_SIZE[0] * TILE_SIZE, height=MAP_SIZE[1] * TILE_SIZE
)
fps_display = pyglet.window.FPSDisplay(window=window)

@window.event
def on_draw():
    window.clear()
    for sprite in curr_map:
        sprite.draw()
    fps_display.draw()


# Set up the event loop and run the game

pyglet.app.run()
