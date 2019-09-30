#!/usr/bin/env python3

import pyglet
from pyglet.window import key
import urllib.request
import json


history_url = "http://192.168.121.144/room/6/history"
info = urllib.request.urlopen(history_url)
data = json.load(info)

print(data)


def load_tile(path, tile_size):
    """Generate an tile from a file."""
    img = pyglet.resource.image(path)
    img.width = tile_size
    img.height = tile_size
    return img


tile_size = 64
images = {
    "Blank": load_tile("empty.png", tile_size),
    "Wall": load_tile("wall.png", tile_size),
    "Doodah": load_tile("star.png", tile_size),
    "SnakeHead": load_tile("yellow_circle.png", tile_size),
    "SnakeBody": load_tile("yellow_ring.png", tile_size),
}


window = pyglet.window.Window(
    width=data[0]["width"] * tile_size,
    height=data[0]["height"] * tile_size,
)


class World:
    """Stores the current world frame."""

    def __init__(self, data: list):
        self.data = data
        self.time_point = 0
        self.tiles = []
        self.width = data[0]["width"]
        self.height = data[0]["height"]

    def load_data(self):
        """Load from `self.data[self.time_point]` into `self.tiles`"""
        world_map = self.data[self.time_point]["tiles"]
        for x in range(self.width):
            for y in range(self.height):
                index = x + y * self.width
                tile = world_map[index]
                # create a sprite and put it in self.tiles
                if tile["type"] == "Wall":
                    sprite = pyglet.sprite.Sprite(
                        images["Wall"],
                        x=x * tile_size, 
                        y=y * tile_size)
                    self.tiles.append(sprite)
    
    def draw(self):
        """Draw the world."""
        for tile in self.tiles:
            tile.draw()


world = World(data)
world.load_data()


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
