use amethyst::prelude::*;
use amethyst::{
    core::*,
    core::{
        math::{Point3, Vector2, Vector3},
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
use crate::components::Tile;

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
        Trans::Switch(Box::new(Awaiting))
    }
}

/// A state representing the game awaiting some input from the player. Waits until the player clicks on a tile or exits.
struct Awaiting;
impl Awaiting {
    fn current_to_move(world: &World) -> Option<Move> {
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
            .filter(|from| !board.is_empty(*from))
            .and_then(|from| {
                board.empty_adjacent(from).map(|to| {
                    Move::new(&*board, from, to)
                })
            })
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
                        if let Some(tile_move) = Awaiting::current_to_move(data.world) {
                            Trans::Push(Box::new(ProcessingMove { tile_move, steps_completed: 0 }))
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
        if world.read_resource::<Board>().is_solved() {
            Trans::Replace(Box::new(Winner {}))
        } else {
            Trans::None
        }
    }
}

#[derive(Debug, Clone)]
struct Move {
    /// The slot we're moving from.
    from: u32,
    /// The slot we're moving to.
    to: u32,
    /// The id of the tile being moved.
    tile: TileId,
    move_step: Vector3<f32>,
}

impl Move {
    const NUM_STEPS: u32 = 10;

    fn new(board: &Board, from: u32, to: u32) -> Self {
        let tile = board.tile_at(from).unwrap();
        let f_pos = board.idx_world(from as i32).unwrap();
        let t_pos = board.idx_world(to as i32).unwrap();

        let move_step: Vector3<f32> = (t_pos - f_pos).scale(1.0 / Move::NUM_STEPS as f32);

        Move { from, to, tile, move_step }
    }
}

/// A state representing the game playing out a move, no input except exiting is accepted..
struct ProcessingMove {
    tile_move: Move,
    steps_completed: u32,
}

impl SimpleState for ProcessingMove {
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        // The only input we care about in this state is the common stuff.
        handle_common_events(data.world, &event).unwrap_or(Trans::None)
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.steps_completed >= Move::NUM_STEPS {
            // Tile has arrived, pop back to awaiting input state.
            data.world.fetch_mut::<Board>().move_tile_at(self.tile_move.from);

            Trans::Pop
        } else {
            // Tile hasn't arrived yet, remain in this state.
            let tiles = data.world.read_component::<Tile>();
            let mut transforms = data.world.write_component::<Transform>();

            for tt in (&tiles, &mut transforms).join() {
                let (tile, transform): (&Tile, &mut Transform) = tt;

                if tile.index == self.tile_move.tile {
                    transform.append_translation(self.tile_move.move_step.clone());
                    self.steps_completed += 1;

                    break;
                }
            }

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
