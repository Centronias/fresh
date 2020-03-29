use amethyst::prelude::*;
use amethyst::{
    assets::*,
    core::*,
    ecs::Entity,
    renderer::*,
    window::*,
    input::{VirtualKeyCode, is_key_down, is_close_requested},
};
use super::components::*;

pub struct Board {

}

impl Board {
    /// Creates and returns an entity representing the board with child entities representing the tiles on the board.
    /// Also adds a Board to the world's storage.
    pub fn init_board(world: &mut World, sprite_sheet: Handle<SpriteSheet>) -> Entity {
        Board::create_board_entity(world, sprite_sheet)
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



