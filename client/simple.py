#!/usr/bin/env python3

import pyglet
from snake import *


# define some constants

TILE_SIZE = 32
MAP_SIZE = (20, 15)


# create our things

snake_part = load_tile("yellow_circle.png", TILE_SIZE)
doodah_img = load_tile("star.png", TILE_SIZE)

snake = Snake(snake_part, snake_part, Position(8, 8, MAP_SIZE), Direction.SOUTH)
doodah = Sprite(Position(5, 2, MAP_SIZE), doodah_img)

window = pyglet.window.Window(
    width=MAP_SIZE[0] * TILE_SIZE, height=MAP_SIZE[1] * TILE_SIZE
)
fps_display = pyglet.window.FPSDisplay(window=window)


# tell our window what to do when events happen

@window.event
def on_draw():
    window.clear()
    doodah.draw()
    snake.draw()
    fps_display.draw()


@window.event
def on_key_press(symbol, modifiers):
    key = pyglet.window.key
    if symbol == key.SPACE:
        snake.grow()
    elif symbol == key.A:
        snake.turn_left()
    elif symbol == key.D:
        snake.turn_right()


# we'll be moving the snake forward every .5 seconds

def move_snake_forward(dt):
    snake.move()

pyglet.clock.schedule_interval(move_snake_forward, 0.5)


# Set up the event loop and run the game

pyglet.app.run()
