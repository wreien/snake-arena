import pyglet
from enum import Enum


def load_tile(path, tile_size):
    """Helper function to load a tile, resizing it as necessary."""
    img = pyglet.resource.image(path)
    img.width = tile_size
    img.height = tile_size
    return img


class Direction(Enum):
    """The direction a snake is facing."""

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


class Position:
    """
    An abstract position on the map.
    Since the map is a torus, this will wrap around.
    This class has no public attributes.
    """

    def __init__(self, x, y, map_size):
        """Create a new position."""
        self._x = x
        self._y = y
        self._map_size = map_size

    def __eq__(self, other):
        return self._x == other._x and self._y == other._y

    def __hash__(self):
        return hash((self._x, self._y))

    def next(self, direction):
        """Returns the position one step in the given direction."""
        width, height = self._map_size
        x, y = {
            Direction.NORTH: (self._x, (self._y + 1) % height),
            Direction.SOUTH: (self._x, (self._y - 1) % height),
            Direction.EAST: ((self._x + 1) % width, self._y),
            Direction.WEST: ((self._x - 1) % width, self._y),
        }.get(direction, (self._x, self._y))
        return Position(x, y, self._map_size)

    def coords(self):
        """Return the real tile position as a 2-tuple (x, y)."""
        return self._x, self._y


class Sprite:
    """
    An image to be drawn on the tile grid.

    pos: The Position of the sprite in the tile grid.
    img: The image resource to draw
    """

    def __init__(self, pos, img):
        """Construct a sprite, setting its parameters."""
        self.pos = pos
        self.img = img

    def draw(self):
        """Draw the sprite."""
        x, y = self.pos.coords()
        self.img.blit(x * self.img.width, y * self.img.height)


class Snake:
    """
    A vicious snake.

    head_img: The image for the head of the snake.
    body_img: The image for the body of the snake.
    parts: A list of 2-tuples for the coordinates of the snake parts.
    direction: The current facing of the snake.
    """

    def __init__(self, head_img, body_img, starting_pos, starting_dir):
        """
        Create the initial beginning of the snake.

        starting_pos: The initial location of the snake in the tile grid.
        starting_dir: The initial direction of the snake in the tile grid.
        """
        self.head_img = head_img
        self.body_img = body_img

        self.parts = [starting_pos]
        self.direction = starting_dir

    def head_position(self):
        """Get the current location of the head."""
        return self.parts[0]

    def turn_left(self):
        """Turn to the left."""
        self.direction = self.direction.left()

    def turn_right(self):
        """Turn to the right."""
        self.direction = self.direction.right()

    def grow(self):
        """
        We've eaten a doodah; grow there.
        This should be next to the current head.
        """
        self.parts.insert(0, self.head_position().next(self.direction))

    def move(self):
        """
        Move to the new position without growing.
        Returns the now vacant position.
        """
        self.parts.insert(0, self.head_position().next(self.direction))
        last_pos = self.parts.pop()
        return last_pos

    def draw(self):
        """Draw the snake."""
        head_spr = Sprite(self.head_position(), self.head_img)
        head_spr.draw()
        for pos in self.parts[1:]:
            body_spr = Sprite(pos, self.body_img)
            body_spr.draw()
