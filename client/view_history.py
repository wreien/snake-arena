#!/usr/bin/env python3

import json
import urllib.request
import sys
import pyglet
from pyglet.window import key

from snake import *

class World:
    """
    Shows off the map.
    """

    def __init__(
        self,
        tile_size: int,
        my_id: int,
        address: str
    ):
        self.images = {
            "Blank": load_tile("empty.png", tile_size),
            "Wall": load_tile("wall.png", tile_size),
            "Doodah": load_tile("star.png", tile_size),

            "Head_North": load_tile("green_arrow_north.png", tile_size),
            "Head_East": load_tile("green_arrow_east.png", tile_size),
            "Head_South": load_tile("green_arrow_south.png", tile_size),
            "Head_West": load_tile("green_arrow_west.png", tile_size),

            "Head_North_me": load_tile("yellow_arrow_north.png", tile_size),
            "Head_East_me": load_tile("yellow_arrow_east.png", tile_size),
            "Head_South_me": load_tile("yellow_arrow_south.png", tile_size),
            "Head_West_me": load_tile("yellow_arrow_west.png", tile_size),

            "Body": load_tile("green_circle.png", tile_size),
            "Body_me": load_tile("yellow_circle.png", tile_size),
            "Tail": load_tile("green_ring.png", tile_size),
            "Tail_me": load_tile("yellow_ring.png", tile_size),
        }
        self.tile_size = tile_size
        self.my_id = my_id
        self.data: list = json.load(urllib.request.urlopen(address))
        self.map = []
        self.map_size = (0, 0)
        self.time_point = 0
        self.step_label = None

    def get_image(self, tile: dict):
        """Given a tile, get the image associated with it (or None)."""
        if tile["type"] in ("Blank", "Wall", "Doodah"):
            return self.images[tile["type"]]
        elif tile["type"] == "SnakeBody":
            name = "Tail" if tile["index"] == 0 else "Body"
            if tile["id"] == self.my_id:
                name = name + "_me"
            return self.images[name]
        elif tile["type"] == "SnakeHead":
            name = "Head_" + tile["dir"]
            if tile["id"] == self.my_id:
                name = name + "_me"
            return self.images[name]

    def get_max_time_point(self):
        return len(self.data) - 1

    def get_world_at_time_point(self, timepoint: int):
        if timepoint < 0 or timepoint > self.get_max_time_point():
            return

        m: dict = self.data[timepoint]
        self.map_size = (m["width"], m["height"])

        tiles = (self.get_image(tile) for tile in m["tiles"])
        positions = (
            Position(x, y, self.map_size)
            for y in range(self.map_size[1])
            for x in range(self.map_size[0])
        )
        self.map = [Sprite(pos, img) for pos, img in zip(positions, tiles)]
        self.step_label = pyglet.text.Label(
            f'{timepoint} / {self.get_max_time_point()}', font_size=32, y=16, x=16)

    def next(self):
        if self.time_point < self.get_max_time_point():
            self.time_point = self.time_point + 1
            self.get_world_at_time_point(self.time_point)

    def prev(self):
        if self.time_point > 0:
            self.time_point = self.time_point - 1
            self.get_world_at_time_point(self.time_point)

    def draw(self):
        for sprite in self.map:
            sprite.draw()
        if self.step_label:
            self.step_label.draw()


room_id = input("Which room? ")
world = World(64, 0, f"http://192.168.121.144/room/{room_id}/history")
if world.get_max_time_point() == 0:
    print("No history.")
    sys.exit(0)
world.get_world_at_time_point(0)


window = pyglet.window.Window(
    width=world.map_size[0] * world.tile_size,
    height=world.map_size[1] * world.tile_size,
)


@window.event
def on_draw():
    world.draw()

@window.event
def on_key_press(symbol, modifiers):
    if symbol == key.RIGHT or symbol == key.SPACE:
        world.next()
    elif symbol == key.LEFT or symbol == key.BACKSPACE:
        world.prev()


pyglet.app.run()
