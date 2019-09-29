#!/usr/bin/env python3

import pyglet
from pyglet.window import key

import asyncio
import json
import sys
import itertools

from snake import *

# get a handle to the asyncio event loop

loop: asyncio.AbstractEventLoop = asyncio.get_event_loop()


# define the world

class World:
    """
    Keeps track of all of the snakes and world grid.

    reader: Read data from the server
    writer: Write data to the server
    tile_size: How large a tile to draw (in pixels)
    my_id: Which snake we are
    map: The tilegrid of walls, snakes, and doodahs
    map_size: The dimensions of the map as (width, height)
    error_text: If there's a problem, paints it on the room
    is_connected: Whether we are still getting updates on the map
    """

    def __init__(
        self,
        reader: asyncio.StreamReader,
        writer: asyncio.StreamWriter,
        tile_size: int,
        my_id: int,
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
        self.reader = reader
        self.writer = writer
        self.my_id = my_id
        self.map = []
        self.map_size = (0, 0)
        self.tile_size = tile_size
        self.error_text = None
        self.is_connected = True
        self.is_sending_data = False

    def set_error_text(self, text):
        """Set the error label message."""
        self.error_text = pyglet.text.Label(text, color=(255,0,0,255), font_size=16)

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


    async def update_world_state(self):
        """Take an update from the server and rebuild the map from it."""
        if not self.is_connected:
            return

        line = await self.reader.readline()
        if not line:
            print("Connection lost!")
            self.set_error_text("Connection lost!")
            self.is_connected = False
            return

        data = json.loads(line)
        if data["state"] == "error":
            print("Error:", data["msg"])
            self.set_error_text("Error: " + data["msg"])
            self.is_connected = False
        elif data["state"] == "done":
            print("Game over!")
            self.set_error_text("Game over!")
            self.is_connected = False
        else:
            if data["state"] == "dead":
                self.set_error_text('Dead!')
            # data["state"] == "playing"

            self.map_size = (data["map"]["width"], data["map"]["height"])
            tiles = (self.get_image(tile) for tile in data["map"]["tiles"])

            # as a generator expression, this is backwards from what you might think
            # also note that we draw bottom-up but we think top-down,
            # so we need to reverse the y-coordinate first
            positions = (
                Position(x, y, self.map_size)
                for y in range(self.map_size[1])
                for x in range(self.map_size[0])
            )
            self.map = [Sprite(pos, img) for pos, img in zip(positions, tiles)]

    async def go(self, direction: str):
        """
        Given a direction 'Left', 'Right', or 'Forward', send it to the server.
        When done update the world state.
        """
        # quit early if we can't do anything
        if self.error_text or not self.is_connected:
            return

        # make sure we only have one request in the pipeline at a time
        if self.is_sending_data:
            return
        self.is_sending_data = True

        self.writer.write(direction.encode("utf-8") + b"\n")
        await self.writer.drain()
        await self.update_world_state()
        self.is_sending_data = False

    def draw(self):
        """Draw the map."""
        for sprite in self.map:
            sprite.draw()
        if self.error_text:
            self.error_text.draw()


# we'll need a connection to the server

async def make_connection(host: str, port: int) -> World:
    print("Connecting...")
    reader, writer = await asyncio.open_connection(host, port)
    writer.write(b'async.py\n')

    print("Waiting for game to start...")
    line = await reader.readline()
    data = json.loads(line.decode("utf-8"))
    my_id = data["id"]

    print("Game started! Loading map and creating window...")
    world = World(reader, writer, 64, my_id)
    await world.update_world_state()
    return world


# do the processing and get things started

world: World = loop.run_until_complete(make_connection("192.168.121.144", 3001))


# Set up the window; we'll just draw for now

window = pyglet.window.Window(
    width=world.map_size[0] * world.tile_size,
    height=world.map_size[1] * world.tile_size,
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
        loop.create_task(world.go("Left"))
    elif symbol == key.RIGHT:
        loop.create_task(world.go("Right"))
    elif symbol == key.UP:
        loop.create_task(world.go("Forward"))


# Set up the event loop and run the game

def poll_loop(dt):
    # see documentation; this has the effect of doing a
    # single poll of the asyncio event loop
    loop.stop()
    loop.run_forever()


def network_refresh_world(dt):
    loop.create_task(world.update_world_state())


pyglet.clock.schedule_interval(poll_loop, 1 / 60)
# pyglet.clock.schedule_interval(network_refresh_world, 1/10)
pyglet.app.run()
