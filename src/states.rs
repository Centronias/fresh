use amethyst::prelude::*;
use amethyst::{
    assets::*,
    core::*,
    ecs::*,
    renderer::*,
    window::*,
    input::{VirtualKeyCode, is_key_down, is_close_requested},
};
use super::components::*;
use super::board::*;

fn load_sprite_sheet(world: &mut World, png_path: &str, ron_path: &str) -> Handle<SpriteSheet> {
    let loader = world.read_resource::<Loader>();

    let texture_handle = loader.load(
        png_path,
        ImageFormat::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>()
    );

    loader.load(
        ron_path,
        SpriteSheetFormat(texture_handle),
        (),
        &world.read_resource::<AssetStorage<SpriteSheet>>(),
    )
}

fn initialise_camera(world: &mut World, parent: Entity) -> Entity {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };

    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(5.0);

    world
        .create_entity()
        .with(camera_transform)
        .with(Parent { entity: parent })
        .with(Camera::standard_2d(width, height))
        .named("camera")
        .build()
}

pub struct Starting;

impl SimpleState for Starting {
    fn on_start(&mut self, data: amethyst::prelude::StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.register::<Named>();
        let background_sprite_sheet_handle = load_sprite_sheet(
            world,
            "Background.png",
            "Background.ron"
        );

        let board = Board::init_board(world, background_sprite_sheet_handle);
        let _camera = initialise_camera(world, board);
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        handle_common_events(data.world, &event).unwrap_or(Trans::None)
    }

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // TODO This is probably a bad way to do it, but assume that once we start updating, loading is done.
        Trans::Switch(Box::new(Awaiting))
    }
}

/// A state representing the game awaiting some input from the player. Waits until the player clicks on a tile or exits.
struct Awaiting;
impl SimpleState for Awaiting {
    fn on_resume(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        // If we resume and the board is solved, the player wins.
        let _boards = _data.world.read_resource::<Board>();
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        handle_common_events(data.world, &event).unwrap_or_else(|| {
            // TODO This is where we'd put the situation where an input has been given and we want to change states.
            Trans::Push(Box::new(ProcessingMove))
        })
    }
}

/// A state representing the game playing out a move, no input except exiting is accepted..
struct ProcessingMove;
impl SimpleState for ProcessingMove {
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        // The only input we care about in this state is the common stuff.
        handle_common_events(data.world, &event)
            .unwrap_or(Trans::None)
    }

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // TODO This is where we'd "simulate" the movement of tiles.

        Trans::Pop
    }
}

fn handle_common_events(
    world: &mut World,
    event: &StateEvent,
) -> Option<SimpleTrans> {
    match event {
        StateEvent::Window(event) => if is_close_requested(event) || is_key_down(event, VirtualKeyCode::Escape) {
            // Player wants to exit.
            Some(Trans::Quit)
        } else if is_key_down(&event, VirtualKeyCode::Space) {
            // Debugging: print the name and transform of all named and transformy entities.
            world.exec(
                |(named, transforms): (ReadStorage<'_, Named>, ReadStorage<'_, Transform>)| {
                    for (name, transform) in (&named, &transforms).join() {
                        println!("{} => {:?}", name.name, transform.translation());
                    }
                },
            );
            None
        } else { None },
        _ => None,
    }
}