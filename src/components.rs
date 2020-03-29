use amethyst::{
    ecs::*,
};

#[derive(Default, Component)]
struct Board {}

#[derive(Component)]
struct Tile {
    _index: u32,
}