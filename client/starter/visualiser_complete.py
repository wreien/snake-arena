#!/usr/bin/env python3

import pyglet
from pyglet.window import key
import urllib.request
import json
import sys


# TODO: load data from history page
history_url = "http://192.168.121.144/room/8/history"
data = urllib.request.urlopen(history_url)
data = json.load(data)


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
                index = x + self.width * y
                tile = world_map[index]
                xpos = x * tile_size
                ypos = y * tile_size
                if tile["type"] == "Wall":
                    sprite = pyglet.sprite.Sprite(images["Wall"], x=xpos, y=ypos)
                elif tile["type"] == "SnakeHead":
                    sprite = pyglet.sprite.Sprite(images["SnakeHead"], x=xpos, y=ypos)
                elif tile["type"] == "SnakeBody":
                    sprite = pyglet.sprite.Sprite(images["SnakeBody"], x=xpos, y=ypos)
                elif tile["type"] == "Doodah":
                    sprite = pyglet.sprite.Sprite(images["Doodah"], x=xpos, y=ypos)
                elif tile["type"] == "Blank":
                    sprite = pyglet.sprite.Sprite(images["Blank"], x=xpos, y=ypos)
                self.tiles.append(sprite)

    def draw(self):
        """Draw the world."""
        for tile in self.tiles:
            tile.draw()

    def next_step(self):
        """Load the next time point."""
        if self.time_point + 1 >= len(self.data):
            print("Error: at last time point")
        else:
            self.time_point = self.time_point + 1
            self.load_frame()


world = World(data)

@window.event
def on_draw():
    """Called to draw the window."""
    window.clear()
    world.draw()

@window.event
def on_key_press(symbol, modifiers):
    """Called to handle key presses."""
    if symbol == key.SPACE:
        world.next_step()


pyglet.app.run()
