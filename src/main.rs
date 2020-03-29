#![warn(
    rust_2018_idioms,
    clippy::all
)]

use amethyst::prelude::*;
use amethyst::{
    core::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
    },
    input::{VirtualKeyCode, is_key_down, is_close_requested},
    ecs::*,
    renderer::*,
    utils::*,
    LoggerConfig,
};

mod states;
use states::*;
mod components;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(LoggerConfig::default());

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("assets");
    let display_config_path = app_root.join("config/display.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?;

    let mut game = Application::build(assets_directory, Starting)?.build(game_data)?;
    game.run();
    Ok(())
}