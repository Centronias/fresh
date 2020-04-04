use super::components::*;
use amethyst::core::math::geometry::Point3;
use amethyst::core::math::Vector3;
use amethyst::prelude::*;
use amethyst::{assets::*, core::*, ecs::Entity, renderer::*};

pub type TileId = u32;

///
/// Slots numbers:
/// + - - +
/// | 2 3 | 1
/// | 0 1 | 0
/// + - - + y
/// x 0 1
///
#[derive(Debug)]
pub struct Board {
    tiles_dim: u32,
    tiles: Vec<Option<TileId>>,
    board_size: f32,
}

impl Board {
    /// Creates and returns an entity representing the board with child entities representing the tiles on the board.
    /// Also adds a Board to the world's storage.
    pub fn init_board(tiles_dim: u32, screen_size: u32, world: &mut World) -> Entity {
        let board = {
            let num_tiles = tiles_dim * tiles_dim;
            let mut tiles = Vec::with_capacity((num_tiles) as usize);
            tiles.push(None);
            for i in 1..num_tiles {
                tiles.push(Some(i));
            }

            let mut board = Board {
                tiles_dim,
                tiles,
                board_size: screen_size as f32,
            };

            board.scramble();
            board
        };

        let ret = board.create_entity(world);

        world.insert(board);

        ret
    }

    fn create_entity(&self, world: &mut World) -> Entity {
        let sprite_sheet = self.load_sprite_sheet(world, "background.jpg");
        let transform = Transform::default();
        let board = world.create_entity().with(transform).named("Board").build();

        for (idx, tile) in self.tiles.iter().enumerate() {
            if let Some(tile_id) = tile {
                self.init_tile(world, sprite_sheet.clone(), *tile_id, idx as u32, board);
            }
        }

        board
    }

    fn init_tile(
        &self,
        world: &mut World,
        sprite_sheet: Handle<SpriteSheet>,
        tile_id: u32,
        index: u32,
        parent: Entity,
    ) -> Entity {
        let (x, y) = self.idx_xy(index);

        let b_bk = Vector3::new(-(self.board_size / 2.0), -(self.board_size / 2.0), 0.0);

        let tile_size = self.tile_size();

        let bk_tk = Vector3::new(x as f32 * tile_size, y as f32 * tile_size, 0.0);

        let tk_tc = Vector3::new(tile_size / 2.0, tile_size / 2.0, 0.0);

        let b_t = b_bk + bk_tk + tk_tc;

        let mut transform = Transform::default();
        transform.set_translation(b_t);
        transform.set_translation_z(-10.0);

        let sprite = SpriteRender {
            sprite_sheet,
            sprite_number: tile_id as usize,
        };
        world
            .create_entity()
            .with(transform)
            .with(Parent { entity: parent })
            .with(sprite)
            .with(Tile { index: tile_id })
            .named(format!("Tile{}", index))
            .build()
    }

    pub fn move_tile_at(&mut self, idx: u32) {
        let to = self.empty_adjacent(idx).unwrap();

        let tiles = &mut self.tiles;

        tiles.swap(to as usize, idx as usize)
    }

    fn scramble(&mut self) {
        // TODO Do something to scramble the board.
        self.move_tile_at(1)
    }

    pub fn tile_at(&self, slot: u32) -> Option<TileId> {
        self.tiles.get(slot as usize).and_then(|it| *it)
    }

    pub fn is_empty(&self, idx: u32) -> bool {
        self.tiles
            .get(idx as usize)
            .map_or(false, |it| it.is_none())
    }

    pub fn empty_adjacent(&self, idx: u32) -> Option<u32> {
        let a = self.idx_xy(idx);
        let b = self.adj_xy(a);

        b.iter()
            .map(|xy| self.xy_idx(*xy))
            .filter(|idx| self.is_empty(*idx))
            .next()
    }

    pub fn is_solved(&self) -> bool {
        let mut it = self.tiles.iter();
        if it
            .next()
            .expect("The first element of tiles should exist")
            .is_some()
        {
            false
        } else {
            let mut expected = 1;
            it.all(|index| {
                if *index == Some(expected) {
                    expected += 1;
                    true
                } else {
                    false
                }
            })
        }
    }

    fn load_sprite_sheet(&self, world: &mut World, png_path: &str) -> Handle<SpriteSheet> {
        let loader = world.read_resource::<Loader>();

        let texture = loader.load(
            png_path,
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        );

        // Texture coordinates are from the top left, but the board tiles are from the bottom left.
        // Invert Y.

        let img_per_tile = 1.0 / self.tiles_dim as f32;

        let sprite_size = (self.tile_size(), self.tile_size());
        let offsets = [0.0; 2];

        let sprite_count = self.tiles_dim * self.tiles_dim;
        let mut sprites = Vec::with_capacity(sprite_count as usize);
        for i in 0..sprite_count {
            let (x, y) = self.idx_xy(i);
            let y = self.tiles_dim - (y + 1);

            let left = img_per_tile * x as f32;
            let right = img_per_tile * (x + 1) as f32;
            let top = img_per_tile * y as f32;
            let bottom = img_per_tile * (y + 1) as f32;

            let sprite = Sprite::from((sprite_size, offsets, [left, right, bottom, top]));

            sprites.push(sprite);
        }

        loader.load_from_data(
            SpriteSheet { texture, sprites },
            (),
            &world.read_resource::<AssetStorage<SpriteSheet>>(),
        )
    }

    pub fn world_idx(&self, loc: Point3<f32>) -> Option<u32> {
        self.world_coord_idx(loc.x, loc.y)
    }

    pub fn idx_world(&self, idx: i32) -> Option<Point3<f32>> {
        // K = Board Corner
        // C = Cursor
        // W / B = Board Center

        self.check_idx(idx).map(|idx| {
            let (x, y) = self.idx_xy(idx);
            let x = x as f32 * self.board_size / self.tiles_dim as f32;
            let y = y as f32 * self.board_size / self.tiles_dim as f32;

            let kxtk = Vector3::new(x, y, 0.0);

            let tkxt = Vector3::new(self.board_size / self.tiles_dim as f32 / 2.0, self.board_size / self.tiles_dim as f32 / 2.0, 0.0);

            // Transform from the bottom left corner (where tile 0 0's corner is) to the center.
            let kxb: Vector3<f32> = Vector3::new(self.board_size / 2.0, self.board_size / 2.0, 0.0);

            let xyz: Vector3<f32> = -kxb + kxtk + tkxt;
            let x = xyz.x;
            let y = xyz.y;
            let z = xyz.z;

            Point3::new(x, y, z)
        })
    }

    /// Turns the given x and y world coordinates into a slot index, if the coordinates correspond to a valid slot.
    fn world_coord_idx(&self, x: f32, y: f32) -> Option<u32> {
        // K = Board Corner
        // C = Cursor
        // W / B = Board Center

        // Transform from the bottom left corner (where tile 0 0's corner is) to the center.
        let kxb: Vector3<f32> = Vector3::new(self.board_size / 2.0, self.board_size / 2.0, 0.0);

        // Transform from the center of the board to the cursor (assuming actual screen coords have been into'd world coords).
        let bxc: Vector3<f32> = Vector3::new(x, y, 0.0);

        // Transform from the bottom left corner to the cursor.
        let kxc: Vector3<f32> = kxb + bxc;

        // Scale the board coordinates to tile coordinates and floor the float values to specific tile coordinates.
        let x: f32 = kxc.x / self.board_size * self.tiles_dim as f32;
        let y: f32 = kxc.y / self.board_size * self.tiles_dim as f32;

        let x = x.floor() as i32;
        let y = y.floor() as i32;

        // Check that the tile coordinates are actually valid, then transform it to a slot index.
        self.check_xy((x, y)).map(|xy| self.xy_idx(xy))
    }

    fn idx_max(&self) -> u32 {
        self.tiles_dim * self.tiles_dim
    }

    fn tile_size(&self) -> f32 {
        self.board_size / self.tiles_dim as f32
    }

    fn adj_xy(&self, (x, y): (u32, u32)) -> Vec<(u32, u32)> {
        let x = x as i32;
        let y = y as i32;

        let it = vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];

        let mut res = Vec::with_capacity(4);
        for xy in it {
            self.check_xy(xy).map(|xy| res.push(xy));
        }

        res
    }

    fn check_x(&self, x: i32) -> Option<u32> {
        if (0i32..(self.tiles_dim as i32)).contains(&x) {
            Some(x as u32)
        } else {
            None
        }
    }

    fn check_y(&self, x: i32) -> Option<u32> {
        if (0i32..(self.tiles_dim as i32)).contains(&x) {
            Some(x as u32)
        } else {
            None
        }
    }

    fn check_xy(&self, (x, y): (i32, i32)) -> Option<(u32, u32)> {
        self.check_x(x)
            .and_then(|x| self.check_y(y).map(|y| (x, y)))
    }

    fn check_idx(&self, idx: i32) -> Option<u32> {
        if (0i32..(self.idx_max() as i32)).contains(&idx) {
            Some(idx as u32)
        } else {
            None
        }
    }

    fn idx_xy(&self, idx: u32) -> (u32, u32) {
        let y = idx / self.tiles_dim;
        let x = idx % self.tiles_dim;

        (x, y)
    }

    fn xy_idx(&self, (x, y): (u32, u32)) -> u32 {
        x + y * self.tiles_dim
    }
}

#[test]
fn checks() {
    let board = Board {
        tiles_dim: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
        board_size: 600.0,
    };

    assert_eq!(board.check_x(1), Some(1));
    assert_eq!(board.check_y(0), Some(0));
    assert_eq!(board.check_idx(3), Some(3));
    assert_eq!(board.check_xy((1, 1)), Some((1, 1)));

    assert_eq!(board.check_x(3), None);
    assert_eq!(board.check_y(4), None);
    assert_eq!(board.check_idx(8), None);
    assert_eq!(board.check_xy((1, 2)), None);

    assert_eq!(board.check_x(-1), None);
    assert_eq!(board.check_y(-2), None);
    assert_eq!(board.check_idx(-2), None);
    assert_eq!(board.check_xy((-1, -2)), None);
    assert_eq!(board.check_xy((-1, -2)), None);
}

#[test]
fn idx_xy() {
    let board = Board {
        tiles_dim: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
        board_size: 600.0,
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
        tiles_dim: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
        board_size: 600.0,
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
fn world_idx() {
    let board = Board {
        tiles_dim: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
        board_size: 600.0,
    };

    assert_eq!(board.world_coord_idx(-200.0, -200.0), Some(0));
    assert_eq!(board.world_coord_idx(200.0, -200.0), Some(1));
    assert_eq!(board.world_coord_idx(-200.0, 200.0), Some(2));
    assert_eq!(board.world_coord_idx(200.0, 200.0), Some(3));
    assert_eq!(board.world_coord_idx(400.0, -200.0), None);
}

#[test]
fn adj() {
    let board = Board {
        tiles_dim: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
        board_size: 600.0,
    };

    let adj = board.adj_xy((0, 0));
    assert_eq!(adj.contains(&(1, 0)), true);
    assert_eq!(adj.contains(&(0, 1)), true);
    assert_eq!(adj.len(), 2);

    let board = Board {
        tiles_dim: 3,
        tiles: vec![
            None, Some(2), Some(1),
            Some(2), Some(2), Some(1),
            Some(2), Some(2), Some(1),
        ],
        board_size: 600.0,
    };

    let adj = board.adj_xy((1, 1));
    assert_eq!(adj.contains(&(1, 0)), true);
    assert_eq!(adj.contains(&(0, 1)), true);
    assert_eq!(adj.contains(&(1, 2)), true);
    assert_eq!(adj.contains(&(2, 1)), true);
    assert_eq!(adj.len(), 4);
}

#[test]
fn idx_world() {
    let board = Board {
        tiles_dim: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
        board_size: 600.0,
    };

    assert_eq!(board.idx_world(0), Some(Point3::new(-150.0, -150.0, 0.0)));
    assert_eq!(board.idx_world(2), Some(Point3::new(-150.0, 150.0, 0.0)));
    assert_eq!(board.idx_world(5), None);
}

#[test]
fn board_solved() {
    let board = Board {
        tiles_dim: 2,
        tiles: vec![None, Some(1), Some(2), Some(3)],
        board_size: 600.0,
    };

    assert_eq!(board.is_solved(), true);
}
