#!/usr/bin/env python3

import asyncio
import json


LEFT = b'Left\n'
RIGHT = b'Right\n'
FORWARD = b'Forward\n'


async def next_state(reader: asyncio.StreamReader) -> dict:
    """Get the next line of data from the server."""
    line = await reader.readline()
    return json.loads(line.decode("utf-8"))


async def make_connection(host: str, port: int):
    """Kick off the whole thing."""
    print("Connecting...")
    reader, writer = await asyncio.open_connection(host, port)
    writer.write(b'My Name Goes Here\n')
    await writer.drain()

    print("Waiting for game to start...")
    data = await next_state(reader)
    my_id = data["id"]

    print("Game started! ID:", my_id)
    # TODO: actually do stuff :)


asyncio.run(make_connection('192.168.121.144', 3001))
