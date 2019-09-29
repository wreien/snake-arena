//! Describe the snake game.

use rand::{distributions::Standard, prelude::*};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};

/// The direction a snake is facing.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    /// Get a new direction to the right of `self`.
    pub fn right(self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    /// Get a new direction to the left of `self`.
    pub fn left(self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
        }
    }
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        use Direction::*;
        const DIRECTIONS: [Direction; 4] = [North, West, South, East];
        DIRECTIONS[rng.gen_range(0, 3)]
    }
}

/// The size of a tile grid.
#[derive(Copy, Clone, Debug, Serialize)]
pub struct Dimensions {
    width: usize,
    height: usize,
}

/// A position in the tile grid.
type Position = (usize, usize);

/// What a tile is filled with.
///
/// Only one of these things can be in a tile at a time.
#[derive(PartialEq, Eq, Copy, Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum Tile {
    /// A snake body, belonging to the snake with given `id`,
    /// where `index = 0` is the tip of the tail
    SnakeBody { id: SnakeID, index: usize },

    /// A snake head, belonging to the snake with given `id`, in given `direction`
    SnakeHead { id: SnakeID, dir: Direction },

    /// A doodah to collect
    Doodah,

    /// A wall that cannot be hit or walked through
    Wall,

    /// Empty space
    Blank,
}

/// An ID for a snake
pub type SnakeID = usize;

/// Keep track of where the snake is and where it's going.
#[derive(Clone, Debug)]
struct Snake {
    /// The direction the snake was last heading.
    pub dir: Direction,

    /// Where the snake's head currently is.
    pub head: Position,

    /// The position of the snake's tail (not including the head).
    ///
    /// `body[0]` is the end of the tail (if it exists), and higher indices
    /// get closer and closer to the `head` position.
    pub body: VecDeque<Position>,
}

impl Snake {
    /// Create a new snake.
    pub fn new(dir: Direction, head: Position) -> Self {
        Snake {
            dir,
            head,
            body: VecDeque::new(),
        }
    }

    /// Get the number of doodahs eaten by this snake.
    pub fn score(&self) -> usize {
        self.body.len()
    }

    /// Move the snake one step in the direction it's facing.
    ///
    /// Returns the spot that has now been freed.
    pub fn step(&mut self, map: Dimensions) -> Position {
        self.body.push_back(self.head);
        self.head = self.next_head_pos(map);
        self.body.pop_front().unwrap()
    }

    /// Grow the snake one step in the direction it's facing.
    ///
    /// This is like move, except the snake doesn't remove
    /// its last segment, and thus nothing is returned.
    pub fn grow(&mut self, map: Dimensions) {
        self.body.push_back(self.head);
        self.head = self.next_head_pos(map);
    }

    /// Get the new head position if the snake were to move.
    pub fn next_head_pos(&self, map: Dimensions) -> Position {
        let (x, y) = self.head;
        let Dimensions { width, height } = map;
        match self.dir {
            Direction::North => (x, (y + 1) % height),
            Direction::South => (x, (y + height - 1) % height),
            Direction::East => ((x + 1) % width, y),
            Direction::West => ((x + width - 1) % width, y),
        }
    }

    /// Test if we have collided with another snake.
    ///
    /// Doesn't test for self-comparison.
    pub fn has_collided(&self, other: &Snake) -> bool {
        self.head == other.head || other.body.iter().any(|&part| part == self.head)
    }

    /// Test if we have collided with ourselves.
    pub fn has_self_collided(&self) -> bool {
        self.body.iter().any(|&part| part == self.head)
    }
}

/// The tile grid.
#[derive(Clone, Debug, Serialize)]
pub struct Map {
    /// Dimensions of the map.
    #[serde(flatten)]
    pub dims: Dimensions,

    /// The tiles occupying the field. This is a representation of a 2d grid.
    pub tiles: Vec<Tile>,

    /// Currently living snakes.
    #[serde(skip)]
    snakes: HashMap<SnakeID, Snake>,

    /// Scores for all snakes in the game.
    pub scores: HashMap<SnakeID, usize>,
}

impl Map {
    /// Create a new map with given `width` and `height`, initialised with the provided
    /// `tiles`.
    ///
    /// # Panics
    ///
    /// The size of the tile map must be the same as `width * height`: that is,
    /// it must cover the whole map. In addition, the only tiles that are permitted are
    /// [`Tile::Wall`] and [`Tile::Blank`]: any other tiles result in a panic.
    ///
    /// [`Tile::Wall`]: enum.Tile.html#variant.Wall
    /// [`Tile::Blank`]: enum.Tile.html#variant.Blank
    pub fn new(
        width: usize,
        height: usize,
        tiles: Vec<Tile>,
        snakes: Vec<SnakeID>,
    ) -> Self {
        assert!(tiles.len() == width * height);
        assert!(tiles.iter().all(|t| t == &Tile::Wall || t == &Tile::Blank));

        let rng = &mut thread_rng();
        let blank_spots = tiles
            .iter()
            .enumerate()
            .filter(|&(_, t)| t == &Tile::Blank)
            .map(|(i, _)| (i % width, i / width))
            .choose_multiple(rng, snakes.len());

        let snakes = snakes
            .into_iter()
            .zip(blank_spots.into_iter().map(|pos| Snake::new(random(), pos)))
            .collect::<HashMap<_, _>>();

        let scores = snakes
            .iter()
            .map(|(id, snake)| (*id, snake.score()))
            .collect();

        let mut me = Map {
            dims: Dimensions { width, height },
            tiles,
            scores,
            snakes,
        };
        me.place_snakes();
        me.place_doodah();

        me
    }

    /// Turn the given snake to the left.
    pub fn turn_left(&mut self, id: SnakeID) {
        if let Some(snake) = self.snakes.get_mut(&id) {
            snake.dir = snake.dir.left();
        }
    }

    /// Turn the given snake to the right.
    pub fn turn_right(&mut self, id: SnakeID) {
        if let Some(snake) = self.snakes.get_mut(&id) {
            print!("Snake {} facing {:?}; ", id, snake.dir);
            snake.dir = snake.dir.right();
            print!("is now facing {:?}\n", snake.dir);
        }
    }

    /// Delete the given snake.
    pub fn delete_snake(&mut self, id: SnakeID) {
        self.snakes.remove(&id);
    }

    /// Test if a snake is still alive.
    pub fn is_alive(&self, id: SnakeID) -> bool {
        self.snakes.get(&id).is_some()
    }

    /// Convert from a position to a tile index.
    fn to_index(&self, (x, y): Position) -> usize {
        x + y * self.dims.width
    }

    /// Get the new map after a time step.
    pub fn step(mut self) -> Result<Self, HashMap<SnakeID, usize>> {
        // rebuild tile map, getting rid of the snakes
        self.cleanup_board();

        // move the snake and see if they got the doodah
        let got_doodah = self.move_snakes();

        // if we're out of snakes, we're done
        if self.snakes.is_empty() {
            return Err(self.scores);
        }

        // fill in the tiles with the still living snakes
        self.place_snakes();

        // fix up the scores
        self.update_scores();

        // replace the doodah if it was picked up
        if let Some(coord) = got_doodah {
            // if it wasn't covered by a snake, get rid of it first
            let idx = self.to_index(coord);
            if let Tile::Doodah = self.tiles[idx] {
                self.tiles[idx] = Tile::Blank;
            }

            // place down a new doodah
            self.place_doodah();
        }

        // return the new details
        Ok(self)
    }

    /// Remove all snake parts from the board
    fn cleanup_board(&mut self) {
        for tile in self.tiles.iter_mut() {
            match tile {
                Tile::SnakeBody { .. } | Tile::SnakeHead { .. } => *tile = Tile::Blank,
                _ => (),
            }
        }
    }

    /// Place all snake parts onto the board
    fn place_snakes(&mut self) {
        for (&id, snake) in self.snakes.iter() {
            let head_idx = self.to_index(snake.head);
            self.tiles[head_idx] = Tile::SnakeHead { id, dir: snake.dir };
            for (index, part) in snake.body.iter().copied().enumerate() {
                let part_idx = self.to_index(part);
                self.tiles[part_idx] = Tile::SnakeBody { id, index };
            }
        }
    }

    /// Move the snakes one step.
    ///
    /// Should be called after `cleanup_board'.
    ///
    /// If a snake got the doodah, returns the doodah's position. Assumes only
    /// one doodah exists at a time.
    fn move_snakes(&mut self) -> Option<Position> {
        // move snakes one step, removing snakes that hit walls
        let mut got_doodah = None;
        let mut snake_copy = std::mem::replace(&mut self.snakes, HashMap::new());
        snake_copy.retain(|_, snake| {
            let new_head = snake.next_head_pos(self.dims);
            let head_idx = self.to_index(new_head);
            match self.tiles.get(head_idx).unwrap() {
                Tile::Doodah => {
                    snake.grow(self.dims);
                    got_doodah = Some(new_head);
                    true
                }
                Tile::Blank => {
                    snake.step(self.dims);
                    true
                }
                Tile::Wall => false,
                _ => panic!("Must call `cleanup_board` first!"),
            }
        });

        // remove snakes that have collided with each other
        self.snakes = snake_copy.clone();
        self.snakes.retain(|id, snake| {
            !snake_copy.iter().any(|(oid, other)| {
                if id == oid {
                    snake.has_self_collided()
                } else {
                    snake.has_collided(other)
                }
            })
        });

        got_doodah
    }

    /// Update the scores for living snakes
    fn update_scores(&mut self) {
        for (&id, snake) in self.snakes.iter() {
            self.scores.insert(id, snake.score());
        }
    }

    /// Place a doodah randomly on a blank tile, if one exists.
    fn place_doodah(&mut self) {
        let new_spot = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, &tile)| tile == Tile::Blank)
            .map(|(i, _)| i)
            .choose(&mut thread_rng());

        // if there's no free spot, don't worry about it
        if let Some(idx) = new_spot {
            self.tiles[idx] = Tile::Doodah;
        }
    }
}
