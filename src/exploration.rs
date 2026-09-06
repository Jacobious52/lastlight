//! Small, persistent discoveries that support decisions between the main sites.
use crate::{
    audio::{AudioCues, Cue},
    gameplay::can_move,
    model::*,
    save,
    world::*,
};
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FindKind {
    Bell,
    Oil,
    Route,
}
pub struct Find {
    pub position: Vec2,
    pub kind: FindKind,
    pub id: usize,
}
pub fn finds(world: &WorldMap) -> Vec<Find> {
    let plan = [
        (1, FindKind::Bell),
        (3, FindKind::Route),
        (5, FindKind::Route),
        (11, FindKind::Route),
        (4, FindKind::Oil),
        (7, FindKind::Oil),
        (9, FindKind::Oil),
        (12, FindKind::Oil),
        (15, FindKind::Oil),
        (17, FindKind::Oil),
        (20, FindKind::Oil),
    ];
    plan.into_iter()
        .enumerate()
        .map(|(id, (room, kind))| {
            let centre = world.rooms[room].center;
            let position = (0..32)
                .map(|i| centre + Vec2::from_angle(i as f32 * std::f32::consts::TAU / 32.) * 115.)
                .find(|p| world.field(*p) > 28. && line_clear(world, centre, *p))
                .unwrap_or(centre + Vec2::X * 60.);
            Find { position, kind, id }
        })
        .collect()
}

pub fn update(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    world: Res<WorldMap>,
    mut game: ResMut<Game>,
    mut cues: ResMut<AudioCues>,
) {
    if game.mode != Mode::Playing || game.map_open {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let interact = keys.just_pressed(KeyCode::KeyR);
    if keys.just_pressed(KeyCode::KeyQ) && game.bell_ready() {
        let mut landing = game.player;
        for i in 1..=24 {
            let next = game.player + game.facing.normalize_or_zero() * i as f32 * 7.;
            if !can_move(&world, next, game.has_veil, game.brightness < 0.1) {
                break;
            }
            landing = next;
        }
        if landing.distance(game.player) > 30. {
            game.bell_origin = game.player;
            game.bell_out = Some(landing);
            game.bell_age = 0.;
        }
    }
    if let Some(p) = game.bell_out {
        let age = game.bell_age;
        game.bell_age += dt;
        if age < 0.65 && game.bell_age >= 0.65 {
            cues.0.push(Cue::Bell);
        }
        if p.distance(game.player) < 48. && game.bell_age > 0.8 {
            if interact {
                game.bell_out = None;
                cues.0.push(Cue::Scuff);
            } else if game.message_time <= 0. {
                game.say("R    retrieve the bell", 2.);
            }
        }
    }
    if let Some(p) = game.anchor
        && p.distance(game.player) < 42.
        && game.anchor_placing <= 0.
        && interact
        && game.bell_out.is_none_or(|b| b.distance(game.player) > 48.)
    {
        game.anchor = None;
        game.anchor_life = 0.;
        cues.0.push(Cue::Scuff);
        game.say("Lantern extinguished", 1.8);
        return;
    }
    for find in finds(&world) {
        if game.collected & (1 << find.id) != 0 || find.position.distance(game.player) > 52. {
            continue;
        }
        if interact {
            match find.kind {
                FindKind::Bell => {
                    game.bell_found = true;
                    game.say("Bell acquired\nQ    throw a sound lure. R    retrieve it when nearby.\nIt distracts creatures without making light.\n\nEnter    continue",9.);
                    game.mode = Mode::Discovery;
                    game.pulse = 0.95;
                    game.target = None;
                }
                FindKind::Oil => {
                    if game.oil >= 2 {
                        game.say("You can carry two oil flasks", 2.);
                        continue;
                    }
                    game.oil += 1;
                    game.say(
                        "Oil flask taken\nYour next placed lantern will burn for one minute.",
                        4.,
                    );
                }
                FindKind::Route => {
                    game.charted |= match find.id {
                        1 => 1,
                        2 => 2,
                        _ => 4,
                    };
                    game.say(
                        "A structure marked in memory\nTab    view the marked locations",
                        4.,
                    );
                }
            }
            game.collected |= 1 << find.id;
            cues.0.push(Cue::Secret);
            save::write_save(&game, &world);
        } else if game.message_time <= 0. {
            game.say(
                match find.kind {
                    FindKind::Bell => "R    take the bell",
                    FindKind::Oil => "R    take an oil flask",
                    FindKind::Route => "R    inspect the route stone",
                },
                2.,
            );
        }
        break;
    }
}
