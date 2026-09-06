mod audio;
mod exploration;
mod gameplay;
mod model;
#[cfg(feature = "playtest")]
mod playtest;
mod render;
mod roots;
mod save;
mod shadow;
mod ui;
mod world;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

fn main() {
    let saved = save::load();
    let seed = save::seed(saved.as_ref());
    let mut world = world::WorldMap::generate(seed);
    let mut game = model::Game::new(seed, world.spawn, world.rooms.len());
    save::restore(saved, &mut game, &mut world);
    #[cfg(feature = "playtest")]
    playtest::apply(&mut game, &mut world);
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(world)
        .insert_resource(game)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Last Light".into(),
                        canvas: Some("#game".into()),
                        resolution: (1280, 800).into(),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins((
            render::RenderPlugin,
            audio::SoundPlugin,
            gameplay::GameplayPlugin,
            ui::InterfacePlugin,
        ))
        .run();
}
