use amethyst::prelude::*;
use amethyst::{
    assets::*,
    core::*,
    ecs::Entity,
    renderer::*,
};
use super::components::*;

static TILES_DIM: u32 = 3;
static BOARD_SIZE: u32 = 600;
static TILE_SIZE: f32 = BOARD_SIZE as f32 / TILES_DIM as f32;

#[derive(Debug)]
pub struct Board {
    height: u32,
    width: u32,
    tiles: Vec<Option<u32>>,
}

fn take_if_in(v: u32, start: u32, end: u32) -> Option<u32> {
    if (start..end).contains(&v) { Some(v) } else { None }
}

impl Board {
    pub fn world_to_idx(&self, loc: amethyst::core::math::geometry::Point3<f32>) -> Option<u32> {
        let mut board_corner_x_board: Transform = Transform::default();
        board_corner_x_board.set_translation_xyz(BOARD_SIZE as f32 / 2.0, BOARD_SIZE as f32 / 2.0, 0.0);

        let mut board_x_cursor = Transform::default();
        board_x_cursor.set_translation_xyz(loc.x, -loc.y, 0.0);

        board_corner_x_board.concat(&board_x_cursor);

        let x = board_corner_x_board.translation().x / BOARD_SIZE as f32 * TILES_DIM as f32;
        let y = board_corner_x_board.translation().y / BOARD_SIZE as f32 * TILES_DIM as f32;
        let x = x as u32;
        let y = y as u32;

        self.check_x(x).and_then(|x|
            self.check_y(y).map(|y|
                self.xy_idx((x, y))
            )
        )
    }

    fn load_sprite_sheet(&self, world: &mut World, png_path: &str) -> Handle<SpriteSheet> {
        let loader = world.read_resource::<Loader>();

        let texture = loader.load(
            png_path,
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>()
        );

        let img_per_tile = 1.0 / TILES_DIM as f32;

        let sprite_size = (TILE_SIZE, TILE_SIZE);
        let offsets = [0.0; 2]; //[(TILE_SIZE as f32) / 2.0; 2];

        let sprite_count = TILES_DIM * TILES_DIM;
        let mut sprites = Vec::with_capacity(sprite_count as usize);
        for i in 0..sprite_count {
            let (x, y) = self.idx_xy(i);

            let left = img_per_tile * x as f32;
            let right = img_per_tile * (x + 1) as f32;
            let top = img_per_tile * y as f32;
            let bottom = img_per_tile * (y + 1) as f32;

            let sprite = Sprite::from((
                sprite_size,
                offsets,
                [left, right, bottom, top]
            ));

            sprites.push(sprite);
        }

        loader.load_from_data(
            SpriteSheet {
                texture,
                sprites,
            },
            (),
            &world.read_resource::<AssetStorage<SpriteSheet>>(),
        )
    }

    pub fn is_solved(&self) -> bool {
        let mut it = self.tiles.iter();
        if *it.next().expect("The first element of tiles should exist") != None {
            false
        } else {
            let mut expected = 1;
            it.all(|index| {
                if *index == Some(expected) {
                    expected += 1;
                    true
                } else { false }
            })
        }
    }

    pub fn scramble(&mut self) {
        // TODO Do something to scramble the board.
        self.tiles.reverse();
    }

    /// Creates and returns an entity representing the board with child entities representing the tiles on the board.
    /// Also adds a Board to the world's storage.
    pub fn init_board(world: &mut World) -> Entity {
        let board = {
            let num_tiles = TILES_DIM * TILES_DIM;
            let mut tiles = Vec::with_capacity((num_tiles) as usize);
            tiles.push(None);
            for i in 1..num_tiles {
                tiles.push(Some(i));
            }

            let mut board = Board {
                height: TILES_DIM,
                width: TILES_DIM,
                tiles,
            };

            board.scramble();
            board
        };

        let ret = board.create_board_entity(world);

        world.insert(board);

        ret
    }

    fn create_board_entity(&self, world: &mut World) -> Entity {
        let sprite_sheet = self.load_sprite_sheet(
            world,
            "background.jpg"
        );

        // TODO Put text on the tiles to make it easier to figure out which is which https://github.com/amethyst/amethyst/blob/master/examples/pong_tutorial_05/pong.rs#L206
        let mut transform = Transform::default();
        transform.set_translation_z(-10.0);
        let board = world
            .create_entity()
            .with(transform)
            .named("Board")
            .build();

        for index in 1..(self.height * self.width) {
            self.init_tile(world, sprite_sheet.clone(), index, board);
        };

        board
    }

    fn init_tile(
        &self,
        world: &mut World,
        sprite_sheet: Handle<SpriteSheet>,
        index: u32,
        parent: Entity
    ) -> Entity {
        let (x, y) = self.idx_xy(index);

        let mut board_x_board_corner = Transform::default();
        board_x_board_corner.set_translation_xyz(-(BOARD_SIZE as f32 / 2.0), -(BOARD_SIZE as f32 / 2.0), 0.0);

        let mut board_corner_x_tile_corner = Transform::default();
        board_corner_x_tile_corner.set_translation_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0);

        let mut tile_corner_x_tile = Transform::default();
        tile_corner_x_tile.set_translation_xyz(TILE_SIZE / 2.0, TILE_SIZE / 2.0, 0.0);

        let mut transform = board_x_board_corner.clone();
        transform.concat(&board_corner_x_tile_corner)
            .concat(&tile_corner_x_tile)
            .set_translation_z(-3.0);

        let sprite = SpriteRender {
            sprite_sheet,
            sprite_number: index as usize,
        };
        world
            .create_entity()
            .with(transform)
            .with(Parent { entity: parent })
            .with(sprite)
            .with(Tile { index })
            .named(format!("Tile{}", index))
            .build()
    }

    fn check_x(&self, x: u32) -> Option<u32> {
        take_if_in(x, 0, self.width)
    }

    fn check_y(&self, y: u32) -> Option<u32> {
        take_if_in(y, 0, self.height)
    }

    fn check_idx(&self, idx: u32) -> Option<u32> {
        take_if_in(idx, 0, self.height * self.width)
    }

    fn idx_xy(&self, idx: u32) -> (u32, u32) {
        let idx = self.check_idx(idx).unwrap();

        let y = idx / self.width;
        let x = idx % self.height;

        (x, y)
    }

    fn xy_idx(&self, (x, y): (u32, u32)) -> u32 {
        let x = self.check_x(x).unwrap();
        let y = self.check_y(y).unwrap();

        x + y * self.width
    }
}

#[test]
fn board_solved() {
    let board = Board {
        height: 2,
        width: 2,
        tiles: vec![None, Some(1), Some(2), Some(3)],
    };

    assert_eq!(board.is_solved(), true);
}

#[test]
fn board_not_solved() {
    let board = Board {
        height: 2,
        width: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
    };

    assert_eq!(board.is_solved(), false);
}

#[test]
fn idx_xy() {
    let board = Board {
        height: 2,
        width: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
    };

    let (x, y) = board.idx_xy(0);
    assert_eq!(x, 0);
    assert_eq!(y, 0);

    let (x, y) = board.idx_xy(1);
    assert_eq!(x, 1);
    assert_eq!(y, 0);

    let (x, y) = board.idx_xy(2);
    assert_eq!(x, 0);
    assert_eq!(y, 1);

    let (x, y) = board.idx_xy(3);
    assert_eq!(x, 1);
    assert_eq!(y, 1);
}

#[test]
fn xy_idx() {
    let board = Board {
        height: 2,
        width: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
    };

    let idx = board.xy_idx((0, 0));
    assert_eq!(idx, 0);

    let idx = board.xy_idx((1, 0));
    assert_eq!(idx, 1);

    let idx = board.xy_idx((0, 1));
    assert_eq!(idx, 2);
    let idx = board.xy_idx((1, 1));

    assert_eq!(idx, 3);
}

#[test]
fn checks() {
    let board = Board {
        height: 2,
        width: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
    };

    assert_eq!(board.check_x(1), Some(1));
    assert_eq!(board.check_y(0), Some(0));
    assert_eq!(board.check_idx(3), Some(3));

    assert_eq!(board.check_x(3), None);
    assert_eq!(board.check_y(4), None);
    assert_eq!(board.check_idx(8), None);
}
