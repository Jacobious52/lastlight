use crate::{model::*, world::*};
use bevy::prelude::*;

pub struct InterfacePlugin;
#[derive(Component)]
enum UiPart {
    Title,
    Menu,
    Hint,
    Place,
    Footer,
}
impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (update, restart));
    }
}
fn setup(mut commands: Commands) {
    for (part, size, color, node) in [
        (
            UiPart::Title,
            34.,
            Color::srgb(0.77, 0.77, 0.77),
            Node {
                left: percent(12.),
                top: percent(31.),
                ..default()
            },
        ),
        (
            UiPart::Menu,
            20.,
            Color::srgb(0.65, 0.65, 0.65),
            Node {
                left: percent(12.),
                top: percent(48.),
                ..default()
            },
        ),
        (
            UiPart::Hint,
            16.,
            Color::srgb(0.65, 0.65, 0.65),
            Node {
                bottom: px(55.),
                left: percent(15.),
                width: percent(70.),
                ..default()
            },
        ),
        (
            UiPart::Place,
            16.,
            Color::srgb(0.49, 0.49, 0.49),
            Node {
                top: px(35.),
                left: px(38.),
                ..default()
            },
        ),
        (
            UiPart::Footer,
            13.,
            Color::srgb(0.39, 0.39, 0.39),
            Node {
                bottom: px(22.),
                left: px(38.),
                ..default()
            },
        ),
    ] {
        let justify = if matches!(part, UiPart::Hint) {
            Justify::Center
        } else {
            Justify::Left
        };
        commands.spawn((
            Text::new(""),
            TextFont {
                font_size: size,
                ..default()
            },
            TextColor(color),
            TextLayout::new_with_justify(justify),
            Node {
                position_type: PositionType::Absolute,
                ..node
            },
            part,
        ));
    }
}
fn update(
    game: Res<Game>,
    world: Res<WorldMap>,
    time: Res<Time>,
    _roots: Res<crate::roots::Rootwork>,
    mut texts: Query<(&UiPart, &mut Text)>,
    mut status_clock: Local<f32>,
    mut frame_average: Local<f32>,
) {
    for (part, mut text) in &mut texts {
        let content = match part {
            UiPart::Title => {
                if game.mode == Mode::Title || (game.mode == Mode::Ending && game.ending_time > 9.)
                {
                    "L A S T   L I G H T".into()
                } else {
                    String::new()
                }
            }
            UiPart::Menu => match game.mode {
                Mode::Title => {
                    if world
                        .sites
                        .iter()
                        .any(|s| s.kind == SiteKind::Ending && s.active)
                    {
                        "View ending   [Enter]".into()
                    } else if game.started && game.elapsed > 0. {
                        "Continue   [Enter]".into()
                    } else {
                        "Begin   [Enter]".into()
                    }
                }
                Mode::Paused => format!("Paused\n\n{}\n\nEnter to return", objective(&game)),
                Mode::Dead => "Extinguished\n\nEnter to return".into(),
                _ => String::new(),
            },
            UiPart::Hint => match game.mode {
                Mode::Discovery => game.message.clone(),
                Mode::Paused => format!(
                    "WASD / arrows    move       Space    pulse       Shift / X    extinguish\n{}\nTab    memory       M    sound {}",
                    abilities(&game),
                    if game.muted { "off" } else { "on" }
                ),
                Mode::Title => {
                    "Wake three structures. Open the passage at the Hollow.\nHeadphones recommended"
                        .into()
                }
                Mode::Dead => "Your discoveries remain".into(),
                Mode::Ending if game.ending_time > 12. => {
                    if game.secrets == 0 {
                        "One small light made it through\n\nThank you for playing".into()
                    } else {
                        format!(
                            "{} small lights made it through\n\nThank you for playing",
                            game.secrets + 1
                        )
                    }
                }
                Mode::Playing if game.map_open => {
                    format!(
                        "{}\n{}\nOnly illuminated places remain.     Tab    return",
                        objective(&game),
                        abilities(&game)
                    )
                }
                Mode::Playing if game.message_time > 0. => game.message.clone(),
                _ => String::new(),
            },
            UiPart::Place => {
                if game.mode == Mode::Playing && game.room_name_time > 0. && !game.map_open {
                    let room = &world.rooms[game.room];
                    format!(
                        "{}\n{}",
                        [
                            "Ash chambers",
                            "Rootwork",
                            "Stone galleries",
                            "Silt reaches"
                        ][room.region as usize],
                        room.name
                    )
                } else {
                    String::new()
                }
            }
            UiPart::Footer if game.test_mode => format!(
                "Development playtest    room {}    saving disabled",
                game.room
            ),
            UiPart::Footer => match game.mode {
                Mode::Title | Mode::Paused | Mode::Ending => {
                    if game.new_confirm {
                        "N again    start a new world".into()
                    } else {
                        format!(
                            "N    new world             M    sound {}             seed {}",
                            if game.muted { "off" } else { "on" },
                            world.seed
                        )
                    }
                }
                Mode::Playing if game.elapsed < 12. => "Esc    pause".into(),
                _ => String::new(),
            },
        };
        // Changing an unchanged Text marks it dirty and needlessly repeats layout.
        if text.0 != content {
            text.0 = content;
        }
    }
    *status_clock += time.delta_secs();
    *frame_average += (time.delta_secs() - *frame_average) * 0.04;
    if *status_clock < 0.25 {
        return;
    }
    *status_clock = 0.;
    #[cfg(target_arch = "wasm32")]
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if let Some(status) = document.get_element_by_id("game-status") {
            status.set_text_content(Some(&format!("{:?}; {}; position {:.0},{:.0}; anchor {}; veil {}; resonators {}; secrets {}; deaths {}; seed {}; frame {:.1}ms",game.mode,world.rooms[game.room].name,game.player.x,game.player.y,game.has_anchor,game.has_veil,game.resonators,game.secrets,game.deaths,world.seed,*frame_average*1000.)));
            let _ = status.set_attribute(
                "data-camera",
                &format!("{:.1},{:.1}", game.camera.x, game.camera.y),
            );
            let _ = status.set_attribute("data-elapsed", &format!("{:.1}", game.elapsed));
            let _ = status.set_attribute("data-danger", &format!("{:.2}", game.danger));
            let _ = status.set_attribute("data-brace", &format!("{:.2}", game.brace));
            let _ = status.set_attribute("data-hurt", &format!("{:.2}", game.hurt_flash));
            let _ = status.set_attribute(
                "data-anchor-placed",
                if game.anchor.is_some() {
                    "true"
                } else {
                    "false"
                },
            );
            let _ = status.set_attribute(
                "data-root-bed",
                if _roots.dragging(game.player) {
                    "true"
                } else {
                    "false"
                },
            );
            for (name, value) in [
                ("anchor-life", format!("{:.1}", game.anchor_life)),
                ("placing", format!("{:.2}", game.anchor_placing)),
                ("snuff", format!("{:.2}", game.anchor_snuff)),
                ("oil", game.oil.to_string()),
                ("charted", game.charted.to_string()),
                (
                    "bell",
                    format!("{}:{}", game.bell_found, game.bell_out.is_some()),
                ),
                (
                    "roots",
                    format!(
                        "{:.2}",
                        _roots
                            .patches
                            .iter()
                            .min_by(|a, b| a
                                .center
                                .distance_squared(game.player)
                                .total_cmp(&b.center.distance_squared(game.player)))
                            .map(|p| p.extension)
                            .unwrap_or(0.)
                    ),
                ),
            ] {
                let _ = status.set_attribute(&format!("data-{name}"), &value);
            }
            let _ = status.set_attribute("data-gait", &format!("{:.1}", game.gait));
            let _ = status.set_attribute("data-brightness", &format!("{:.2}", game.brightness));
            let _ = status.set_attribute("data-speed", &format!("{:.1}", game.ground_speed));
            let _ = status.set_attribute("data-pulse", &format!("{:.2}", game.pulse));
            let _ = status.set_attribute("data-moved", &format!("{:.1}", game.moved));
        }
        if let Some(loading) = document.get_element_by_id("loading") {
            let _ = loading.set_attribute("hidden", "");
        }
    }
}

fn objective(game: &Game) -> String {
    if game.resonators >= 3 {
        "The passage at the Hollow is open. Return there.".into()
    } else {
        format!(
            "Open the passage at the Hollow\nStructures awake: {} / 3",
            game.resonators
        )
    }
}

fn abilities(game: &Game) -> String {
    let mut lines = vec!["R    inspect / take / hold to wake a structure".to_string()];
    if game.has_anchor {
        lines.push(format!(
            "E    set down a lantern (stand still)    R    extinguish nearby\nOil flasks: {} / 2",
            game.oil
        ));
    }
    if game.bell_found {
        lines.push("Q    throw the bell    R    retrieve nearby".into());
    }
    if game.has_veil {
        lines.push("Veil: extinguish to cross woven curtains".into());
    }
    lines.join("\n")
}
fn restart(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    mut world: ResMut<WorldMap>,
    mut creatures: Query<&mut Creature>,
) {
    if !matches!(game.mode, Mode::Title | Mode::Paused | Mode::Ending)
        || !keys.just_pressed(KeyCode::KeyN)
    {
        return;
    }
    if !game.new_confirm {
        game.new_confirm = true;
        return;
    }
    let seed = world
        .seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
        % 1_000_000;
    let test_mode = game.test_mode;
    *world = WorldMap::generate(seed);
    *game = Game::new(seed, world.spawn, world.rooms.len());
    game.test_mode = test_mode;
    game.started = true;
    crate::save::write_save(&game, &world);
    for mut c in &mut creatures {
        c.home = world.creature_spawn(c.room);
        c.position = c.home;
        c.alert = 0.;
        c.velocity = Vec2::ZERO;
    }
}
