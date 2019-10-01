#!/usr/bin/env python3

import asyncio
import json
import random
import sys
import enum


LEFT = b'Left\n'
RIGHT = b'Right\n'
FORWARD = b'Forward\n'


class Direction(enum.Enum):
    """A cardinal direction."""

    NORTH = 0
    EAST = 1
    SOUTH = 2
    WEST = 3

    def right(self):
        """Get the direction after turning right."""
        return Direction((self.value + 1) % 4)

    def left(self):
        """Get the direction after turning left."""
        return Direction((self.value - 1) % 4)

    @classmethod
    def from_str(cls, s: str):
        """Create a direction from a string."""
        if s == "North":
            return cls.NORTH
        if s == "East":
            return cls.EAST
        if s == "South":
            return cls.SOUTH
        if s == "West":
            return cls.WEST
        return None


class World:
    """
    Wrap the world in a more easily-accessible manner.

    width: The width of the map
    height: The height of the map
    tiles: The (raw) list of tiles in the map, row-major.
    """
    def __init__(self, data):
        self.width = data["width"]
        self.height = data["height"]
        self.tiles = data["tiles"]

    def get_tile(self, x: int, y: int) -> dict:
        """Get the tile at the given position. Wraps around."""
        x = x % self.width
        y = y % self.height
        return self.tiles[x + y * self.width]

    def pos_in_dir(self, x: int, y: int, direction: Direction) -> (int, int):
        """Find the position of the tile in the given direction."""
        if direction == Direction.NORTH:
            y = y + 1
        elif direction == Direction.SOUTH:
            y = y - 1
        elif direction == Direction.EAST:
            x = x + 1
        elif direction == Direction.WEST:
            x = x - 1
        return (x % self.width, y % self.height)

    def tile_in_dir(self, x: int, y: int, direction: Direction) -> dict:
        """Get the tile in the given direction from the point."""
        return self.get_tile(*self.pos_in_dir(x, y, direction))

    def find_tile_pos(self, **kwargs) -> (int, int):
        """Find the position of the first tile with the given qualifiers."""
        for x in range(self.width):
            for y in range(self.height):
                t = self.tiles[x + y * self.width]
                for key, value in kwargs.items():
                    if t[key] != value:
                        break
                else:
                    return (x, y)
        return None


def safe_tile(tile) -> bool:
    """Determines if the given tile is safe to move onto."""
    if tile["type"] == "Blank":
        return True
    if tile["type"] == "Doodah":
        return True
    if tile["type"] == "SnakeTail" and tile["seg"] == 0:
        return True
    return False


def decision(my_id: int, world: World) -> str:
    """Pick a direction that won't kill us, if it exists"""
    # first we need to find ourselves
    pos = world.find_tile_pos(type="SnakeHead", id=my_id)
    if not pos:
        print("Couldn't find snake head!")
        sys.exit(1)
    x, y = pos
    direction = Direction.from_str(world.get_tile(x, y)["dir"])

    # now we see what choices we have
    choices = []
    if safe_tile(world.tile_in_dir(x, y, direction)):
        choices.append(FORWARD)
    if safe_tile(world.tile_in_dir(x, y, direction.left())):
        choices.append(LEFT)
    if safe_tile(world.tile_in_dir(x, y, direction.right())):
        choices.append(RIGHT)

    # if we have any, pick one at random, otherwise just go forward
    if len(choices) > 0:
        return random.choice(choices)
    else:
        return FORWARD


async def next_state(reader: asyncio.StreamReader) -> dict:
    """Get the next lot of data from the server."""
    line = await reader.readline()
    return json.loads(line.decode("utf-8"))


async def loop(reader: asyncio.StreamReader, writer: asyncio.StreamWriter, my_id: int):
    """The main event loop. Gets the current world and returns the decision made."""
    data = await next_state(reader)
    while data["state"] == "playing":
        choice = decision(my_id, World(data["map"]))
        writer.write(choice)
        await writer.drain()
        data = await next_state(reader)


async def make_connection(host: str, port: int):
    """Kick the whole thing off."""
    print("Connecting...")
    reader, writer = await asyncio.open_connection(host, port)
    writer.write(b'ai_nocollide.py\n')  # say who we are
    await writer.drain()

    print("Waiting for game to start...")
    data = await next_state(reader)
    my_id = data["id"]
    assert data["state"] == "start"

    print("Game started!")
    await loop(reader, writer, my_id)


# do it forever
while True:
    try:
        asyncio.run(make_connection('192.168.121.144', 3001))
    except KeyboardInterrupt:
        break
    except:
        # sleep a bit before trying again
        asyncio.wait(asyncio.sleep(10))
