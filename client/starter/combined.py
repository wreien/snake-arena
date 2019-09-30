#!/usr/bin/env python3

import pyglet
from pyglet.window import key

import asyncio
import json
import sys


LEFT = b'Left\n'
RIGHT = b'Right\n'
FORWARD = b'Forward\n'


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


# manually handle the asyncio event loop
loop = asyncio.AbstractEventLoop = asyncio.get_event_loop()

async def next_state(reader: asyncio.StreamReader) -> dict:
    """Get the next line of data from the server."""
    line = await reader.readline()
    return json.loads(line.decode("utf-8"))

async def make_connection(host: str, port: int):
    """Get connected to the server."""
    print("Connecting...")
    reader, writer = await asyncio.open_connection(host, port)
    writer.write(b'My Name Goes Here\n')
    await writer.drain()

    print("Waiting for game to start...")
    data = await next_state(reader)
    my_id = data["id"]

    print("Game started!")
    # TODO: return a class to manage the server interaction
    return my_id

my_id = loop.run_until_complete(make_connection('192.168.121.144', 3001))
print("My ID:", my_id)

window = pyglet.window.Window(
    width=640,
    height=480,
)


@window.event
def on_draw():
    """Called to draw the window."""
    window.clear()


@window.event
def on_key_press(symbol, modifiers):
    """Called to handle key presses."""
    pass


def poll_loop(dt):
    """Keep the asyncio event loop rolling."""
    # see documentation; this has the effect of doing a single
    # poll of the event loop, even though it looks weird ;)
    loop.stop()
    loop.run_forever()


pyglet.clock.schedule_interval(poll_loop, 1 / 60)
pyglet.app.run()
