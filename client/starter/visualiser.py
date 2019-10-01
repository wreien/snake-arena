#!/usr/bin/env python3

import pyglet
from pyglet.window import key
import urllib.request
import json
import sys


# TODO: load data from history page
data = []


# if there's no data, quit immediately
if len(data) == 0:
    print("No history to view!")
    sys.exit(0)


def load_tile(path, tile_size):
    """Generate an tile from a file."""
    img = pyglet.resource.image(path)
    img.width = tile_size
    img.height = tile_size
    return img


# our available images to use
tile_size = 64
images = {
    "Blank": load_tile("empty.png", tile_size),
    "Wall": load_tile("wall.png", tile_size),
    "Doodah": load_tile("star.png", tile_size),
    "SnakeHead": load_tile("yellow_circle.png", tile_size),
    "SnakeBody": load_tile("yellow_ring.png", tile_size),
}


# the window we use
window = pyglet.window.Window(
    width=data[0]["width"] * tile_size,
    height=data[0]["height"] * tile_size,
)


class World:
    """Stores the current world frame for a given time point."""

    def __init__(self, data: list):
        self.data = data
        self.time_point = 0
        self.tiles = []
        self.width = data[0]["width"]
        self.height = data[0]["height"]
        # load in the first frame
        self.load_frame()

    def load_frame(self):
        """Load from `self.data[self.time_point]` into `self.tiles`"""
        world_map = self.data[self.time_point]["tiles"]
        self.tiles = []
        for x in range(self.width):
            for y in range(self.height):
                # TODO: load into self.tiles
                pass

    def draw(self):
        """Draw the world."""
        for tile in self.tiles:
            tile.draw()

    # TODO control time_point


world = World(data)


@window.event
def on_draw():
    """Called to draw the window."""
    window.clear()
    world.draw()


@window.event
def on_key_press(symbol, modifiers):
    """Called to handle key presses."""
    pass


pyglet.app.run()
