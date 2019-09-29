#!/usr/bin/env python3

import asyncio
import json
import random


# the bulk of our AI

LEFT = b'Left\n'
RIGHT = b'Right\n'
FORWARD = b'Forward\n'

def decision(my_id, world) -> str:
    # we'll just pick entirely randomly
    return random.choice([LEFT, RIGHT, FORWARD])


async def next_state(reader: asyncio.StreamReader) -> dict:
    """Get the next lot of data from the server."""
    line = await reader.readline()
    return json.loads(line.decode("utf-8"))


async def loop(reader: asyncio.StreamReader, writer: asyncio.StreamWriter, my_id: int):
    """The main event loop. Gets the current world and returns the decision made."""
    data = await next_state(reader)
    while data["state"] == "playing":
        choice = decision(my_id, data["map"])
        writer.write(choice)
        await writer.drain()
        data = await next_state(reader)


async def make_connection(host: str, port: int):
    """Kick the whole thing off."""
    print("Connecting...")
    reader, writer = await asyncio.open_connection(host, port)
    writer.write(b'ai_random.py\n')  # say who we are
    await writer.drain()

    print("Waiting for game to start...")
    data = await next_state(reader)
    my_id = data["id"]
    assert data["state"] == "start"

    print("Game started!")
    await loop(reader, writer, my_id)


asyncio.run(make_connection('192.168.121.144', 3001))
