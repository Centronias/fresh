use amethyst::prelude::*;
use amethyst::{
    core::*,
    core::{
        math::{Point3, Vector2},
        Named, Parent, Transform,
    },
    ecs::{Entity, Join},
    input::{is_close_requested, is_key_down, InputHandler, StringBindings},
    input::{InputEvent, VirtualKeyCode},
    renderer::camera::Camera,
    window::ScreenDimensions,
    winit::MouseButton,
};

use super::board::*;

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

        let board = Board::init_board(4, 600, world);
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
impl Awaiting {
    fn current_cursor_as_valid_board_idx(world: &World) -> Option<u32> {
        let input = world.read_resource::<InputHandler<StringBindings>>();
        let dimensions = world.read_resource::<ScreenDimensions>();
        let cameras = world.read_storage::<Camera>();
        let transforms = world.read_storage::<Transform>();

        let mouse_position = input
            .mouse_position()
            .expect("Mouse should be on the screen if we're detecting the keypress");

        let ct: (&Camera, &Transform) = (&cameras, &transforms)
            .join()
            .next()
            .expect("Didn't find the camera");
        let (camera, transform) = ct;

        let dimensions = &*dimensions;
        let screen_dims = Vector2::new(dimensions.width(), dimensions.height());

        let pos: Point3<f32> = camera.projection().screen_to_world_point(
            Point3::new(
                mouse_position.0,
                mouse_position.1,
                transform.translation().z,
            ),
            screen_dims,
            transform,
        );

        let board = world.read_resource::<Board>();
        board
            .world_idx(pos)
            .filter(|idx| !board.is_empty(*idx) && board.empty_adjacent(*idx).is_some())
    }
}

impl SimpleState for Awaiting {
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        handle_common_events(data.world, &event).unwrap_or_else(|| match event {
            StateEvent::Input(input_event) => match input_event {
                InputEvent::MouseButtonReleased(mouse_button) => match mouse_button {
                    MouseButton::Left => {
                        if let Some(idx) = Awaiting::current_cursor_as_valid_board_idx(data.world) {
                            dbg!(idx);
                            // TODO Only transition if there's an open slot adjacent to the selected tile.
                            Trans::Push(Box::new(ProcessingMove { _idx: idx }))
                        } else {
                            Trans::None
                        }
                    }
                    _ => Trans::None,
                },
                _ => Trans::None,
            },
            _ => Trans::None,
        })
    }

    fn update(
        &mut self,
        StateData { world, .. }: &mut StateData<'_, GameData<'_, '_>>,
    ) -> SimpleTrans {
        // TODO If we resume and the board is solved, the player wins.
        let board = world.read_resource::<Board>();

        if board.is_solved() {
            Trans::Replace(Box::new(Winner {}))
        } else {
            Trans::None
        }
    }
}

/// A state representing the game playing out a move, no input except exiting is accepted..
struct ProcessingMove {
    _idx: u32,
}
impl SimpleState for ProcessingMove {
    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        println!("Starting processing move!");
        // TODO We know which slot on the board is moving, and we can determine which slot on the board is empty.
        //  Put together a vector for the move
        //  Figure out a constant number of updates over which to perform the move
        //  Divide the move vector by that number of updates...
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        // The only input we care about in this state is the common stuff.
        handle_common_events(data.world, &event).unwrap_or(Trans::None)
    }

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // TODO This is where we'd "simulate" the movement of tiles.
        //  ... and then apply the update step with each update.
        //  Once we get close enough, decide that we're done and pop back.
        if true {
            // Tile has arrived, pop back to awaiting input state.
            Trans::Pop
        } else {
            // Tile hasn't arrived yet, remain in this state.
            Trans::None
        }
    }
}

struct Winner {}
impl SimpleState for Winner {
    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        print!("Ye win");
        unimplemented!()
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        // The only input we care about in this state is the common stuff.
        handle_common_events(data.world, &event).unwrap_or(Trans::None)
    }
}

fn handle_common_events<T>(world: &mut World, event: &StateEvent) -> Option<Trans<T, StateEvent>> {
    use ecs::*;

    match event {
        StateEvent::Window(event) => {
            if is_close_requested(event) || is_key_down(event, VirtualKeyCode::Escape) {
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

                let board = world.read_resource::<Board>();
                println!("Board => {:?}", &*board);

                None
            } else {
                None
            }
        }
        _ => None,
    }
}
