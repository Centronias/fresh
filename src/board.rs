use amethyst::prelude::*;
use amethyst::{
    assets::*,
    core::*,
    ecs::Entity,
    renderer::*,
};
use super::components::*;

pub struct Board {
    height: usize,
    width: usize,
    tiles: Vec<Option<usize>>,
}

impl Board {
    pub fn is_solved(&self) -> bool {
        let mut it = self.tiles.iter();
        if *it.next().expect("The first element of tiles should exist") != None {
            false
        } else {
            let mut expected = 1usize;
            it.all(|index| {
                if *index == Some(expected) {
                    expected += 1;
                    true
                } else { false }
            })
        }
    }

    /// Creates and returns an entity representing the board with child entities representing the tiles on the board.
    /// Also adds a Board to the world's storage.
    pub fn init_board(world: &mut World, sprite_sheet: Handle<SpriteSheet>) -> Entity {
        Board::create_board_resource(world);
        Board::create_board_entity(world, sprite_sheet)
    }

    fn create_board_resource(world: &mut World) {
        let height = 2;
        let width = 2;

    let mut tiles = Vec::new();
    tiles.push(None);
    for i in 1..height * width {
    tiles.push(Some(i));
    }

    world.insert(Board {
    height,
    width,
    tiles,
    });
    }

    fn create_board_entity(world: &mut World, sprite_sheet: Handle<SpriteSheet>) -> Entity {
        // TODO Put text on the tiles to make it easier to figure out which is which https://github.com/amethyst/amethyst/blob/master/examples/pong_tutorial_05/pong.rs#L206
        let mut transform = Transform::default();
        transform.set_translation_z(-10.0);
        let sprite = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: 0,
        };
        let board = world
            .create_entity()
            .with(transform)
            .with(sprite)
            .named("Board")
            .build();

        for index in 0usize..4 {
            Board::init_tile(world, sprite_sheet.clone() , index, board);
        };

        board
    }

    fn init_tile(world: &mut World, sprite_sheet: Handle<SpriteSheet>, index: usize, parent: Entity) -> Entity {
        let mut transform = Transform::default();
        transform.set_translation_xyz((index as f32) * 120.0, 0.0, -3.0);
        let sprite = SpriteRender {
            sprite_sheet,
            sprite_number: index + 1,
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
}

#[cfg(test)]
fn board_solved() {
    let board = Board {
        height: 2,
        width: 2,
        tiles: vec![None, Some(1), Some(2), Some(3)],
    };

    assert_eq!(board.is_solved(), true);
}

#[cfg(test)]
fn board_not_solved() {
    let board = Board {
        height: 2,
        width: 2,
        tiles: vec![None, Some(2), Some(1), Some(3)],
    };

    assert_eq!(board.is_solved(), false);
}