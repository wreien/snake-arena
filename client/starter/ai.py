#!/usr/bin/env python3

from __future__ import annotations
import asyncio
import json
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

    def right(self) -> Direction:
        """Get the direction after turning right."""
        return Direction((self.value + 1) % 4)

    def left(self) -> Direction:
        """Get the direction after turning left."""
        return Direction((self.value - 1) % 4)

    @classmethod
    def from_str(cls, s: str) -> cls:
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
    def __init__(self, data: dict):
        self.width = data["width"]
        self.height = data["height"]
        self.tiles = data["tiles"]

    def get_tile(self, x: int, y: int) -> dict:
        """Get the tile at the given position. Wraps around."""
        # TODO
        pass

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
        return get_tile(*pos_in_dir(x, y, direction))

    def find_tile_pos(self, **kwargs) -> (int, int):
        """
        Find the position of the first tile with the given qualifiers.

        Pass the qualifiers in as key-value pairs; for example, to find snake heads
        with my id call `world.find_tile_pos(type="SnakeHead", id=my_id)`
        """
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
    # TODO
    return True


def decision(my_id: int, world: World) -> str:
    """Pick a direction that won't kill us, if it exists"""
    # first we need to find ourselves
    pos = world.find_tile_pos(type="SnakeHead", id=my_id)
    if not pos:
        print("Couldn't find snake head!")
        sys.exit(1)
    x, y = pos
    direction = Direction.from_str(world.get_tile(x, y)["dir"])

    # TODO: make a "real" decision
    return FORWARD


async def next_state(reader: asyncio.StreamReader) -> dict:
    """Get the next line of data from the server."""
    line = await reader.readline()
    return json.loads(line.decode("utf-8"))


async def event_loop(my_id: int,
                     reader: asyncio.StreamReader,
                     writer: asyncio.StreamWriter):
    """Run the decision loop."""
    # TODO: actually do something :)
    pass


async def run_ai(host: str, port: int):
    """Kick off the whole thing."""
    print("Connecting...")
    reader, writer = await asyncio.open_connection(host, port)
    writer.write(b'My Name Goes Here\n')
    await writer.drain()

    print("Waiting for game to start...")
    data = await next_state(reader)
    my_id = data["id"]

    print("Game started! ID:", my_id)
    await event_loop(my_id, reader, writer)


# automatically restart the AI when its done
while True:
    asyncio.run(run_ai('192.168.121.144', 3001))
