//! CC0 recorded effects and the licensed Suno instrumental score.
use std::collections::HashMap;

use bevy::{
    audio::{AudioSinkPlayback, SpatialScale, Volume},
    prelude::*,
};

use crate::{
    model::{Creature, CreatureKind, Game, Mode, line_clear},
    world::{SiteKind, WorldMap},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    Pulse,
    Anchor,
    Ability,
    Resonator,
    Secret,
    Death,
    Step,
    Wake,
    Ending,
    Begin,
    Scuff,
    Hurt,
    Distress,
    Bell,
}

#[derive(Resource, Default)]
pub struct AudioCues(pub Vec<Cue>);

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioCues>()
            .init_resource::<SoundState>()
            .add_systems(Startup, create_sounds)
            .add_systems(PostUpdate, soundscape);
    }
}

#[derive(Resource)]
struct SoundBank {
    effects: Vec<Handle<AudioSource>>,
    creatures: Vec<Handle<AudioSource>>,
    warning: Handle<AudioSource>,
    music: Vec<Handle<AudioSource>>,
    steps: Vec<Handle<AudioSource>>,
    scuff: Handle<AudioSource>,
}

#[derive(Resource, Default)]
struct SoundState {
    started: bool,
    clock: f32,
    next_music: f32,
    music_played: bool,
    music_was_active: bool,
    music_track: usize,
    diagnostic_clock: f32,
    next_step: f32,
    step_index: usize,
    next_wake: f32,
    next_anchor: f32,
    next_distress: f32,
    creature_times: HashMap<Entity, f32>,
    creature_noticed: HashMap<Entity, bool>,
}

#[derive(Component)]
struct Ears;

#[derive(Component)]
struct SoundEmitter(Entity);

#[derive(Component)]
struct MusicTrack;

#[derive(Component, Clone, Copy)]
enum Voice {
    Music(f32),
    Effect(f32),
}

fn create_sounds(mut commands: Commands, server: Res<AssetServer>) {
    let effects = [
        "pulse",
        "anchor",
        "ability",
        "resonator",
        "secret",
        "death",
        "begin",
        "wake",
        "ending",
        "begin",
        "begin",
        "hurt",
        "distress",
        "bell",
    ]
    .map(|name| server.load(format!("audio/effects/{name}.ogg")))
    .to_vec();
    let creatures = (0..8)
        .map(|i| server.load(format!("audio/effects/creature-{i}.ogg")))
        .collect();
    let music = vec![
        server.load("audio/stone-and-breath.ogg"),
        server.load("audio/stone-and-breath-refuge.ogg"),
    ];
    commands.insert_resource(SoundBank {
        effects,
        steps: ["stone", "sand"]
            .into_iter()
            .flat_map(|surface| (0..6).map(move |take| format!("audio/foley/{surface}-{take}.ogg")))
            .map(|path| server.load(path))
            .collect(),
        scuff: server.load("audio/foley/scuff.ogg"),
        creatures,
        warning: server.load("audio/effects/warning.ogg"),
        music,
    });
    // The same world-to-audio scale is supplied on every spatial voice, so this
    // remains independent of camera zoom and the application's AudioPlugin.
    commands.spawn((Ears, SpatialListener::new(180.), Transform::default()));
}

#[allow(clippy::too_many_arguments)]
fn soundscape(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Game>,
    world: Res<WorldMap>,
    bank: Res<SoundBank>,
    assets: Res<Assets<AudioSource>>,
    mut state: ResMut<SoundState>,
    mut cues: ResMut<AudioCues>,
    creatures: Query<(Entity, &Creature)>,
    mut ears: Query<&mut Transform, With<Ears>>,
    mut emitters: Query<(&mut Transform, &SoundEmitter), Without<Ears>>,
    mut voices: Query<(&mut AudioSink, &Voice)>,
    mut spatial: Query<(&mut SpatialAudioSink, &Voice)>,
    tracks: Query<Entity, With<MusicTrack>>,
) {
    let dt = time.delta_secs().min(0.1);
    let playing = game.mode == Mode::Playing;
    let paused = matches!(game.mode, Mode::Paused | Mode::Title);
    if !paused {
        state.clock += dt;
    }
    for mut transform in &mut ears {
        transform.translation = game.player.extend(0.);
    }
    for (mut transform, emitter) in &mut emitters {
        if let Ok((_, creature)) = creatures.get(emitter.0) {
            transform.translation = creature.position.extend(0.);
        }
    }

    if !state.started && (playing || cues.0.contains(&Cue::Begin)) {
        state.started = true;
        state.next_music = state.clock + 1.;
    }

    // Recorded music gives way to quiet; no generated pressure/noise loop.
    let sanctuary = world.sites.iter().any(|site| {
        site.kind == SiteKind::Sanctuary
            && site.position.distance_squared(game.player) < 170. * 170.
    });
    let fade = (dt * 2.5).min(1.);
    for (mut sink, voice) in &mut voices {
        let target = match *voice {
            Voice::Music(gain) => {
                gain * (1. - game.danger.clamp(0., 1.) * 0.55)
                    * if game.mode == Mode::Ending {
                        (1. - game.ending_time / 3.).clamp(0., 1.)
                    } else {
                        1.
                    }
            }
            Voice::Effect(gain) => gain,
        };
        let target = if game.muted { 0. } else { target };
        let current = sink.volume().to_linear();
        sink.set_volume(Volume::Linear(current + (target - current) * fade));
        if paused {
            sink.pause();
        } else {
            sink.play();
        }
    }
    for (mut sink, voice) in &mut spatial {
        if let Voice::Effect(gain) = *voice {
            sink.set_volume(Volume::Linear(if game.muted { 0. } else { gain }));
        }
        if paused {
            sink.pause();
        } else {
            sink.play();
        }
    }

    if playing && game.danger > 0.63 && state.clock >= state.next_distress {
        cues.0.push(Cue::Distress);
        state.next_distress = state.clock + 0.85;
    }
    // Gameplay drains into a bounded queue; mute never accumulates stale cues.
    for cue in cues.0.drain(..).take(12) {
        if game.muted {
            continue;
        }
        if cue == Cue::Step {
            if state.clock < state.next_step {
                continue;
            }
            state.next_step = state.clock + 0.25;
        }
        if cue == Cue::Wake {
            if state.clock < state.next_wake {
                continue;
            }
            state.next_wake = state.clock + 1.7;
        }
        if cue == Cue::Anchor {
            if state.clock < state.next_anchor {
                continue;
            }
            state.next_anchor = state.clock + 0.35;
        }
        let gain = match cue {
            Cue::Pulse => 0.47,
            Cue::Anchor => 0.35,
            Cue::Ability => 0.56,
            Cue::Resonator => 0.70,
            Cue::Secret => 0.37,
            Cue::Death => 0.54,
            Cue::Step => {
                if game.brightness < 0.1 {
                    0.18
                } else {
                    0.48
                }
            }
            Cue::Scuff => 0.22,
            Cue::Hurt => 0.95,
            Cue::Distress => 0.75,
            Cue::Wake => 0.33,
            Cue::Ending => 0.67,
            Cue::Begin => 0.14,
            Cue::Bell => 0.62,
        };
        if state.music_played && matches!(cue, Cue::Ability | Cue::Resonator | Cue::Ending) {
            // Discoveries can postpone a piece but never bring it forward into
            // an already playing long composition.
            state.next_music = state.next_music.max(state.clock + 38.);
        }
        let pitch = if cue == Cue::Step {
            0.99 + (state.clock * 19.31).sin() * 0.025
        } else {
            1.0
        };
        let source = if cue == Cue::Step {
            let surface = if world.rooms[game.room].region == 3 {
                6
            } else {
                0
            };
            let index = surface + state.step_index % 6;
            state.step_index += 1;
            bank.steps[index].clone()
        } else if cue == Cue::Scuff {
            bank.scuff.clone()
        } else {
            bank.effects[cue as usize].clone()
        };
        if cue == Cue::Bell {
            commands.spawn((
                AudioPlayer::new(source),
                PlaybackSettings::DESPAWN
                    .with_volume(Volume::Linear(gain))
                    .with_spatial(true)
                    .with_spatial_scale(SpatialScale::new_2d(1. / 350.)),
                Transform::from_translation(game.bell_out.unwrap_or(game.player).extend(0.)),
                Voice::Effect(gain),
            ));
            continue;
        }
        commands.spawn((
            AudioPlayer::new(source),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(gain))
                .with_speed(pitch),
            Voice::Effect(gain),
        ));
    }

    let music_active = !tracks.is_empty();
    if state.music_was_active && !music_active {
        // Count the quiet interval from actual completion, not the spawn time.
        state.next_music = state.next_music.max(state.clock + 24.);
    }
    state.music_was_active = music_active;
    state.diagnostic_clock += dt;
    if state.diagnostic_clock >= 0.5 {
        state.diagnostic_clock = 0.;
        #[cfg(target_arch = "wasm32")]
        if let Some(status) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("game-status"))
        {
            let sink = voices
                .iter()
                .find(|(_, voice)| matches!(voice, Voice::Music(_)));
            let (label, position, gain) = match sink {
                Some((sink, _)) => (
                    if sink.is_paused() {
                        "paused"
                    } else {
                        "playing"
                    },
                    sink.position().as_secs_f32(),
                    sink.volume().to_linear(),
                ),
                None => (if music_active { "loading" } else { "waiting" }, 0., 0.),
            };
            let _ = status.set_attribute("data-music", label);
            let _ = status.set_attribute("data-music-seconds", &format!("{position:.1}"));
            let _ = status.set_attribute("data-music-gain", &format!("{gain:.2}"));
            let _ = status.set_attribute("data-footsteps", &state.step_index.to_string());
            let _ = status.set_attribute(
                "data-music-file",
                if state.music_track == 0 {
                    "stone-and-breath.ogg"
                } else {
                    "stone-and-breath-refuge.ogg"
                },
            );
        }
    }
    if !playing {
        return;
    }
    if music_due(state.clock, state.next_music, music_active, game.muted) {
        let index = if state.music_played {
            usize::from(sanctuary)
        } else {
            0
        };
        // Wait for the file instead of silently consuming its scheduling slot.
        if assets.contains(&bank.music[index]) {
            let gain = 0.82;
            commands.spawn((
                AudioPlayer::new(bank.music[index].clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
                Voice::Music(gain),
                MusicTrack,
            ));
            state.music_played = true;
            state.music_track = index;
            state.music_was_active = true;
        }
    }

    let clock = state.clock;
    let mut spatial_voices = spatial.iter().len();
    for (entity, creature) in &creatures {
        let distance = creature.position.distance(game.player);
        let audible_range = if creature.kind == CreatureKind::Leviathan {
            1450.
        } else {
            900.
        };
        if distance > audible_range {
            continue;
        }
        let noticed = state.creature_noticed.entry(entity).or_insert(false);
        let warning = creature.notice > 0.05 && !*noticed && creature.kind != CreatureKind::Grazer;
        *noticed = creature.notice > 0.01;
        let next = state
            .creature_times
            .entry(entity)
            .or_insert(clock + 0.7 + creature.phase.rem_euclid(2.7));
        if clock < *next && !warning {
            continue;
        }
        let (index, interval, gain) = match creature.kind {
            CreatureKind::Listener => (0, 5.3 - creature.alert.clamp(0., 1.) * 1.2, 0.72),
            CreatureKind::Still => (1, 5.6, 0.54),
            CreatureKind::Grazer => (2, 4.7, 0.34),
            CreatureKind::Leviathan => (3, 12.5, 0.82),
        };
        *next = clock
            + (if warning { 3.8 } else { interval })
            + (creature.phase + clock * 0.13).sin().abs() * 1.3;
        if game.muted {
            continue;
        }
        if spatial_voices >= 7 {
            continue;
        }
        // A Still makes friction when it moves, falling quiet under direct
        // observation. Its changing rhythm is an imperfect sense in the dark.
        if !warning
            && creature.kind == CreatureKind::Still
            && creature.velocity.length_squared() < 4.
        {
            continue;
        }
        let edge_fade = (1. - distance / audible_range).clamp(0., 1.).sqrt();
        let obstructed = !line_clear(&world, game.player, creature.position);
        let gain = gain * edge_fade * if obstructed { 0.66 } else { 1.0 };
        let index = index + if obstructed { 4 } else { 0 };
        commands.spawn((
            AudioPlayer::new(if warning && creature.kind == CreatureKind::Listener {
                bank.warning.clone()
            } else {
                bank.creatures[index].clone()
            }),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(gain))
                .with_spatial(true)
                .with_spatial_scale(SpatialScale::new_2d(1. / 350.))
                .with_speed(0.95 + creature.alert.clamp(0., 1.) * 0.13),
            Transform::from_translation(creature.position.extend(0.)),
            SoundEmitter(entity),
            Voice::Effect(gain),
        ));
        spatial_voices += 1;
    }
    // Generation/restarts can replace the ecosystem without retaining old IDs.
    if state.creature_times.len() > creatures.iter().len() + 32 {
        state
            .creature_times
            .retain(|entity, _| creatures.contains(*entity));
    }
}

fn music_due(clock: f32, next: f32, active: bool, muted: bool) -> bool {
    clock >= next && !active && !muted
}
