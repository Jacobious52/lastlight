use crate::{
    audio::{AudioCues, Cue},
    model::*,
    save,
    world::*,
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::roots::Rootwork>()
            .add_systems(Startup, spawn_creatures)
            .add_systems(
                Update,
                (
                    controls,
                    crate::roots::update,
                    crate::exploration::update,
                    movement,
                    mechanisms,
                    ecosystem,
                    progression,
                    housekeeping,
                )
                    .chain(),
            );
    }
}

pub fn spawn_creatures(mut commands: Commands, world: Res<WorldMap>) {
    let cast = [
        (2, CreatureKind::Listener),
        (4, CreatureKind::Grazer),
        (5, CreatureKind::Still),
        (8, CreatureKind::Listener),
        (9, CreatureKind::Grazer),
        (11, CreatureKind::Still),
        (12, CreatureKind::Listener),
        (13, CreatureKind::Still),
        (14, CreatureKind::Listener),
        (15, CreatureKind::Grazer),
        (16, CreatureKind::Still),
        (17, CreatureKind::Listener),
        (19, CreatureKind::Leviathan),
        (20, CreatureKind::Still),
        (21, CreatureKind::Grazer),
        (22, CreatureKind::Listener),
    ];
    for (i, (room, kind)) in cast.into_iter().enumerate() {
        if world.rooms.get(room).is_some() {
            let position = world.creature_spawn(room);
            commands.spawn(Creature {
                kind,
                position,
                home: position,
                velocity: Vec2::ZERO,
                alert: 0.,
                phase: i as f32 * 1.74,
                gait: 0.,
                last_known: position,
                sense_age: 20.,
                notice: 0.,
                facing: 1.,
                room,
            });
        }
    }
}

// Independent ECS input resources keep input and game state access explicit.
#[allow(clippy::too_many_arguments)]
fn controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    world: Res<WorldMap>,
    mut game: ResMut<Game>,
    mut cues: ResMut<AudioCues>,
    mut dark_toggle: Local<bool>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        game.muted = !game.muted;
    }
    if keys.just_pressed(KeyCode::Escape) {
        game.mode = match game.mode {
            Mode::Playing => Mode::Paused,
            Mode::Paused => Mode::Playing,
            other => other,
        };
        game.target = None;
        game.map_open = false;
    }
    if matches!(
        game.mode,
        Mode::Title | Mode::Dead | Mode::Paused | Mode::Discovery
    ) && (keys.just_pressed(KeyCode::Enter) || mouse.just_pressed(MouseButton::Left))
    {
        if game.mode == Mode::Discovery {
            game.message_time = 0.;
        }
        if game.mode == Mode::Dead {
            game.player = game.checkpoint;
            game.camera = game.player;
            game.danger = 0.;
            game.brace = 0.;
            game.invulnerable = 4.;
            game.anchor = None;
            game.anchor_placing = 0.;
            game.anchor_life = 0.;
            game.bell_out = None;
            game.pulse = 1.3;
        }
        if !matches!(game.mode, Mode::Paused | Mode::Discovery) {
            *dark_toggle = false;
        }
        game.mode = Mode::Playing;
        game.started = true;
        game.target = None;
        game.new_confirm = false;
        if let Some(ending) = world
            .sites
            .iter()
            .find(|s| s.kind == SiteKind::Ending && s.active)
        {
            game.mode = Mode::Ending;
            game.player = ending.position;
            game.camera = ending.position;
            game.ending_time = 0.;
            cues.0.push(Cue::Ending);
        } else {
            cues.0.push(Cue::Begin);
        }
        return;
    }
    if game.mode != Mode::Playing {
        return;
    }
    if keys.just_pressed(KeyCode::Tab) {
        game.map_open = !game.map_open;
        game.target = None;
    }
    if game.map_open {
        return;
    }
    if keys.just_pressed(KeyCode::KeyX) || mouse.just_pressed(MouseButton::Right) {
        *dark_toggle = !*dark_toggle;
    }
    let dark =
        keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) || *dark_toggle;
    let desired = if dark { 0.015 } else { 0.58 };
    game.brightness +=
        (desired - game.brightness) * (1. - (-16. * time.delta_secs().min(0.05)).exp());
    if (keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::KeyF))
        && game.pulse_cooldown <= 0.
    {
        game.pulse = 1.65;
        game.pulse_cooldown = 3.8;
        game.pulses += 1;
        cues.0.push(Cue::Pulse);
    }
    if keys.just_pressed(KeyCode::KeyE) && game.lantern_ready() {
        game.anchor_placing = 1.15;
        let beside = game.player + game.facing.normalize_or_zero() * 18.;
        game.anchor_origin = if can_move(&world, beside, game.has_veil, game.brightness < 0.1) {
            beside
        } else {
            game.player
        };
        game.anchor_charge = 2.5;
        game.target = None;
        game.velocity = Vec2::ZERO;
    }
    if (mouse.pressed(MouseButton::Left) || mouse.just_pressed(MouseButton::Left))
        && let Ok(window) = windows.single()
        && let Some(cursor) = window.cursor_position()
    {
        #[cfg(target_arch = "wasm32")]
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("game-status"))
        {
            let _ = el.set_attribute(
                "data-input",
                &format!(
                    "{:.0},{:.0};{:.0},{:.0};{}",
                    cursor.x,
                    cursor.y,
                    window.width(),
                    window.height(),
                    window.scale_factor()
                ),
            );
        }
        let relative = (cursor - Vec2::new(window.width(), window.height()) * 0.5) * VIEW_HEIGHT
            / window.height();
        let target = game.camera + Vec2::new(relative.x, -relative.y);
        game.target = Some(target);
    }
}

pub fn can_move(world: &WorldMap, p: Vec2, veil: bool, dark: bool) -> bool {
    if world.field(p) < 13. {
        return false;
    }
    !world.gates.iter().any(|gate| {
        if gate.open > 0.82 || gate.latched {
            return false;
        }
        if gate.kind == GateKind::Veil && veil && dark {
            return false;
        }
        let relative = p - gate.position;
        relative.dot(gate.normal).abs() < 22.
            && relative.perp_dot(gate.normal).abs() < gate.half_width + 12.
    })
}
fn movement(
    mut cues: ResMut<AudioCues>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    world: Res<WorldMap>,
    roots: Res<crate::roots::Rootwork>,
    mut game: ResMut<Game>,
) {
    if game.mode != Mode::Playing || game.map_open {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let mut direction = Vec2::new(
        (keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight)) as u8 as f32
            - (keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)) as u8 as f32,
        (keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)) as u8 as f32
            - (keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)) as u8 as f32,
    );
    if direction.length_squared() > 0. {
        game.target = None;
    } else if let Some(target) = game.target {
        if target.distance(game.player) < 10. {
            game.target = None;
        } else {
            direction = (target - game.player).normalize_or_zero();
        }
    }
    if direction.length_squared() > 0. {
        game.anchor_placing = 0.;
        direction = direction.normalize();
        game.facing = direction;
    }
    game.impact *= (-dt * 12.).exp();
    game.impact_cooldown = (game.impact_cooldown - dt).max(0.);
    if direction.x.abs() > 0.15 {
        game.body_facing = direction.x.signum();
    }
    let previous_speed = game.velocity.length();
    let root = roots.dragging(game.player);
    let speed = (if game.brightness < 0.1 { 62. } else { 82. }) * if root { 0.64 } else { 1. };
    game.velocity = game.velocity.lerp(
        direction * speed,
        1. - (-if direction == Vec2::ZERO { 9. } else { 4.8 } * dt).exp(),
    );
    // Restore a saved contact to nearby free floor before integrating movement.
    // A changing membrane cannot push this recovery through a sealed passage.
    if world.field(game.player) < 13. {
        for radius in [4., 8., 12., 18., 26., 38., 54.] {
            let safe = (0..24)
                .map(|i| {
                    game.player + Vec2::from_angle(i as f32 * std::f32::consts::TAU / 24.) * radius
                })
                .find(|p| can_move(&world, *p, game.has_veil, game.brightness < 0.1));
            if let Some(p) = safe {
                game.player = p;
                break;
            }
        }
    }
    let delta = game.velocity * dt;
    let before = game.player;
    game.player = slide_move(&world, before, delta, game.has_veil, game.brightness < 0.1);
    let travelled = before.distance(game.player);
    game.ground_speed += (travelled / dt.max(0.001) - game.ground_speed) * (1. - (-dt * 14.).exp());
    let old_step = (game.gait / 38.).floor();
    game.moved += travelled;
    if game.ground_speed >= 8. {
        game.gait += travelled;
    }
    if (game.gait / 38.).floor() > old_step && travelled > 0.01 {
        cues.0.push(Cue::Step);
        game.foot_echo = if world.rooms[game.room].region == 2 {
            if game.brightness < 0.1 { 0.25 } else { 1. }
        } else {
            0.
        };
        game.impact = if game.brightness < 0.1 { 0.22 } else { 0.48 };
    }
    // Finish the current half-step into a planted pose when movement stops.
    // This advances an opaque pose, never a blend of two silhouettes, and does
    // not create extra walking sounds while standing against an obstruction.
    if game.ground_speed < 8. && travelled < 0.15 {
        let contact = (game.gait / 38.).ceil() * 38.;
        let remaining = contact - game.gait;
        game.gait = if remaining < 0.2 {
            contact
        } else {
            game.gait + remaining * (1. - (-dt * 18.).exp())
        };
    }
    let stopped = direction == Vec2::ZERO && previous_speed > 18. && game.velocity.length() <= 18.;
    let blocked =
        delta.length() > 1. && travelled < delta.length() * 0.25 && game.ground_speed > 15.;
    if (stopped || blocked) && game.impact_cooldown <= 0. {
        cues.0.push(Cue::Scuff);
        game.impact = if blocked { 0.8 } else { 0.35 };
        game.impact_cooldown = 0.8;
    }
    let room = world.room_at(game.player);
    let idle_target = if game.ground_speed < 8. { 1. } else { 0. };
    game.idle += (idle_target - game.idle) * (1. - (-dt * 4.).exp());
    if room != game.room {
        game.room = room;
        game.room_name_time = 4.;
    }
    if game.brightness > 0.15 || game.pulse > 0. {
        game.visited[room] = true;
    }
}

/// Small swept steps follow rough wall tangents while retaining the destination.
/// A closed membrane still blocks every candidate; opening it resumes a held
/// direction or a previous click without needing another key press.
fn slide_move(world: &WorldMap, start: Vec2, delta: Vec2, veil: bool, dark: bool) -> Vec2 {
    let steps = (delta.length() / 4.).ceil().max(1.) as usize;
    let step = delta / steps as f32;
    let mut position = start;
    for _ in 0..steps {
        if can_move(world, position + step, veil, dark) {
            position += step;
            continue;
        }
        let gradient = Vec2::new(
            world.field(position + Vec2::X * 3.) - world.field(position - Vec2::X * 3.),
            world.field(position + Vec2::Y * 3.) - world.field(position - Vec2::Y * 3.),
        )
        .normalize_or_zero();
        let tangent = step - gradient * step.dot(gradient).min(0.);
        let mut best = Vec2::ZERO;
        for offset in [tangent, Vec2::new(step.x, 0.), Vec2::new(0., step.y)] {
            if offset.length_squared() > best.length_squared()
                && offset.dot(step) > 0.
                && can_move(world, position + offset, veil, dark)
            {
                best = offset;
            }
        }
        // A shallow glancing contact follows the wall instead of requiring a
        // new key press. Every candidate is swept and still respects membranes.
        if best.length_squared() < step.length_squared() * 0.08
            && world.field(position + step) < 13.
        {
            for angle in [
                0.35_f32, -0.35, 0.70, -0.70, 1.05, -1.05, 1.40, -1.40, 1.57, -1.57,
            ] {
                let candidate = Vec2::from_angle(angle).rotate(step);
                if can_move(world, position + candidate, veil, dark) {
                    best = candidate;
                    break;
                }
            }
        }
        position += best;
    }
    position
}

fn mechanisms(
    time: Res<Time>,
    mut world: ResMut<WorldMap>,
    mut game: ResMut<Game>,
    mut cues: ResMut<AudioCues>,
) {
    if game.mode != Mode::Playing || game.map_open {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    for gate in &mut world.gates {
        let distance = gate.position.distance(game.player);
        let anchor = game
            .anchor
            .is_some_and(|a| a.distance(gate.position) < 180.);
        let opening = match gate.kind {
            GateKind::Light => (game.pulse > 0.3 && distance < 260.) || anchor,
            GateKind::Shade => {
                (distance < 330. && game.brightness < 0.09 && game.pulse < 0.15 && !anchor)
                    || distance > 430.
            }
            GateKind::Anchor => anchor,
            GateKind::Veil => game.has_veil && game.brightness < 0.1 && distance < 180.,
            GateKind::Final => game.resonators >= 3,
        };
        if gate.kind == GateKind::Light && opening && !gate.latched {
            gate.latched = true;
            cues.0.push(Cue::Wake);
        }
        // The held-light passage becomes a physical shortcut after passing its centre.
        if gate.kind == GateKind::Anchor && gate.open > 0.84 && distance < 35. {
            gate.latched = true;
        }
        // A membrane waits for feet to leave its throat before closing.
        let relative = game.player - gate.position;
        let occupied = gate.open > 0.82
            && relative.dot(gate.normal).abs() < 42.
            && relative.perp_dot(gate.normal).abs() < gate.half_width + 15.;
        let target = if opening || gate.latched || occupied {
            1.
        } else {
            0.
        };
        gate.open = (gate.open + (target - gate.open) * dt * 4.).clamp(0., 1.);
    }
    if game.pulse > 0.4 {
        let room = world.room_at(game.player);
        game.visited[room] = true;
    }
}

fn ecosystem(
    time: Res<Time>,
    world: Res<WorldMap>,
    mut game: ResMut<Game>,
    mut creatures: Query<&mut Creature>,
    mut cues: ResMut<AudioCues>,
) {
    if game.mode != Mode::Playing || game.map_open {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let mut danger: f32 = 0.;
    let mut contact = false;
    let mut consumed = false;
    let player = game.player;
    let sanctuary = world.sites.iter().any(|s| {
        (s.kind == SiteKind::Sanctuary && s.position.distance(player) < 155.)
            || (s.kind == SiteKind::Resonator && s.active && s.position.distance(player) < 95.)
    });
    for mut c in &mut creatures {
        c.phase += dt;
        let d = c.position.distance(player);
        let active = d < 1400. || c.position.distance(c.home) > 50.;
        if !active {
            continue;
        }
        let visible =
            d < (if game.pulse > 0.2 {
                660.
            } else {
                60. + game.brightness * 265.
            }) && line_clear(&world, player, c.position);
        let seen = (visible && (game.brightness > 0.1 || game.pulse > 0.2))
            || game.anchor.is_some_and(|a| {
                a.distance(c.position) < 250. && line_clear(&world, a, c.position)
            });
        let signal = if game.pulse > 0.35 && d < 960. && line_clear(&world, player, c.position) {
            1.0
        } else if game.brightness > 0.25 && d < 380. && line_clear(&world, player, c.position) {
            0.8
        } else {
            0.0
        };
        c.sense_age += dt;
        if c.kind == CreatureKind::Still && seen {
            c.last_known = game.anchor.unwrap_or(player);
            c.sense_age = 0.;
        }
        let hears_steps = (c.kind == CreatureKind::Still && game.ground_speed > 20. && d < 260.
            || c.kind == CreatureKind::Listener && game.foot_echo > 0.3 && d < 430.)
            && line_clear(&world, player, c.position);
        let senses = (signal > 0. || hears_steps) && !sanctuary;
        if senses {
            c.last_known = player;
            c.sense_age = 0.;
            c.notice = (c.notice + dt).min(2.);
        } else {
            c.notice = (c.notice - dt * 0.65).max(0.);
        }
        c.alert = (c.alert + dt * if senses { 0.65 } else { -0.48 }).clamp(0., 1.);
        if let Some(bell) = game.bell_out
            && (0.65..4.5).contains(&game.bell_age)
            && c.kind != CreatureKind::Grazer
            && c.position.distance(bell) < 470.
            && line_clear(&world, c.position, bell)
        {
            c.last_known = bell;
            c.sense_age = 0.;
        }
        let anchor_target = game
            .anchor
            .filter(|a| a.distance(c.position) < 600. && line_clear(&world, *a, c.position));
        if c.kind == CreatureKind::Leviathan {
            // Its root is beyond the cavern. The searching front permeates
            // stone; a completed deep structure drives it back for the return.
            let repelled = world.sites.iter().any(|s| s.room == 22 && s.active);
            let lure = game.anchor.filter(|a| a.distance(c.home) < 850.);
            let target = if repelled {
                c.home + Vec2::Y * 270.
            } else if let Some(a) = lure {
                a
            } else if c.sense_age < 5. {
                c.last_known
            } else {
                c.home + Vec2::Y * 100.
            };
            let delta = target - c.position;
            let desired = delta.normalize_or_zero() * (delta.length() / 100.).min(1.) * 27.;
            c.velocity = c.velocity.lerp(desired, 1. - (-dt * 1.8).exp());
            let velocity = c.velocity;
            c.position += velocity * dt;
            let edge = crate::shadow::distance(&c, player);
            danger = danger.max((1. - edge.max(0.) / 160.).max(0.) * 0.72);
            if edge < 9. && game.invulnerable <= 0. && !sanctuary {
                contact = true;
            }
            if game
                .anchor
                .is_some_and(|a| crate::shadow::distance(&c, a) < 12.)
            {
                consumed = true;
            }
            continue;
        }
        let patrol = c.home + Vec2::new((c.phase * 0.19).cos() * 85., (c.phase * 0.26).sin() * 65.);
        let investigating = c.sense_age < 4.8 && !sanctuary && c.position.distance(c.home) < 460.;
        let search = if investigating { c.last_known } else { patrol };
        // Detection begins with an audible inhale and a pause. Losing a signal
        // leaves a remembered location, never an invisible tether to the player.
        let (target, speed) = match c.kind {
            CreatureKind::Listener => (
                anchor_target.unwrap_or(search),
                if c.room == 2 {
                    32.
                } else if c.notice > 1.15 && investigating {
                    66.
                } else if c.notice > 0.05 && c.notice < 1.15 {
                    8.
                } else if investigating || anchor_target.is_some() {
                    43.
                } else {
                    20.
                },
            ),
            CreatureKind::Still => (
                search,
                if seen {
                    0.
                } else if investigating {
                    49.
                } else {
                    19.
                },
            ),
            CreatureKind::Grazer => (
                anchor_target.unwrap_or(patrol),
                if anchor_target.is_some() { 36. } else { 17. },
            ),
            CreatureKind::Leviathan => (
                anchor_target.unwrap_or(search),
                if investigating && c.notice > 1.7 {
                    54.
                } else {
                    14.
                },
            ),
        };
        // The first organism remains within its teaching alcove by choosing a
        // reachable bounded target, never by teleporting against its velocity.
        let target = if c.room == 2 && target.distance(c.home) > 220. {
            c.home + (target - c.home).normalize_or_zero() * 220.
        } else {
            target
        };
        // Arrive smoothly instead of overshooting and reversing every frame.
        // Feel the solid geometry ahead and steer along it before collision.
        let offset = target - c.position;
        let arrive = (offset.length() / 95.).clamp(0., 1.);
        let mut direction = offset.normalize_or_zero();
        if speed > 0. && world.field(c.position + direction * 42.) < 20. {
            let e = 7.;
            let gradient = Vec2::new(
                world.field(c.position + Vec2::X * e) - world.field(c.position - Vec2::X * e),
                world.field(c.position + Vec2::Y * e) - world.field(c.position - Vec2::Y * e),
            )
            .normalize_or_zero();
            direction = (direction + gradient * 1.8).normalize_or_zero();
        }
        let desired = direction * speed * arrive;
        let damping = if speed == 0. { 13. } else { 5.0 };
        c.velocity = c.velocity.lerp(desired, 1. - (-dt * damping).exp());
        let step = c.velocity * dt;
        let before = c.position;
        if c.velocity.x.abs() > 5. {
            c.facing = c.velocity.x.signum();
        }
        for offset in [Vec2::new(step.x, 0.), Vec2::new(0., step.y)] {
            let next = c.position + offset;
            // Creatures obey the same sealed passages, preventing attacks through barriers.
            if can_move(&world, next, false, false) {
                c.position = next;
            } else if offset.x != 0. {
                c.velocity.x *= (-dt * 9.).exp();
            } else {
                c.velocity.y *= (-dt * 9.).exp();
            }
        }
        c.gait += before.distance(c.position);
        if let Some(a) = game.anchor
            && c.position.distance(a) < 32.
        {
            consumed = true;
            c.alert = 0.;
        }
        if c.kind != CreatureKind::Grazer && !sanctuary {
            let proximity = (1. - d / 250.).max(0.);
            danger = danger.max(proximity * (0.25 + c.alert * 0.6));
            let radius = if c.kind == CreatureKind::Leviathan {
                54.
            } else {
                27.
            };
            if d < radius && game.invulnerable <= 0. {
                contact = true;
            }
        }
        // The first organism cannot leave its observation chamber or kill on first contact.
        if c.room == 2 && d < 32. && !game.has_anchor {
            game.invulnerable = 2.;
            game.say("Extinguish. Let it pass.", 3.);
        }
    }
    if consumed {
        game.anchor_snuff += dt;
        if game.anchor_snuff > 1.4 {
            game.anchor = None;
            game.anchor_life = 0.;
            game.anchor_charge = 2.;
            game.anchor_snuff = 0.;
            cues.0.push(Cue::Scuff);
        }
    } else {
        game.anchor_snuff = (game.anchor_snuff - dt * 2.).max(0.);
    }
    if contact && game.brace <= 0. {
        // First contact gives a readable stumble and time to escape. Returning
        // into contact before the lantern steadies remains dangerous.
        game.brace = 7.;
        game.invulnerable = 1.5;
        game.danger = 0.6;
        game.impact = 1.4;
        game.hurt_flash = 1.;
        game.anchor_placing = 0.;
        cues.0.push(Cue::Hurt);
        contact = false;
    }
    let rate = if contact {
        0.85
    } else if game.brace > 0. {
        -0.15
    } else {
        -1.1
    };
    game.danger = (game.danger + rate * dt).clamp(danger * 0.6, 1.);
    if game.danger >= 0.999 {
        game.mode = Mode::Dead;
        game.deaths += 1;
        game.target = None;
        game.velocity = Vec2::ZERO;
        cues.0.push(Cue::Death);
        save::write_save(&game, &world);
    }
}

fn progression(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<WorldMap>,
    mut game: ResMut<Game>,
    mut cues: ResMut<AudioCues>,
) {
    if game.mode != Mode::Playing || game.map_open {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let player = game.player;
    let mut charging = false;
    let mut changed = false;
    for site in &mut world.sites {
        let d = site.position.distance(player);
        if site.kind == SiteKind::Sanctuary {
            if d < 65. && game.checkpoint.distance(site.position) > 5. {
                game.checkpoint = site.position;
                site.active = true;
                game.invulnerable = 2.;
                game.say("Sanctuary", 3.);
                changed = true;
            }
            continue;
        }
        if site.active {
            continue;
        }
        if d < 220. && game.message_time <= 0. && game.brightness > 0.1 {
            match site.kind {
                SiteKind::AnchorAbility | SiteKind::VeilAbility => {
                    game.say("Come closer. Release a pulse.", 3.)
                }
                SiteKind::Resonator if site.room == 16 => game.say(
                    "Leave a lantern nearby. Extinguish at the centre. Hold R.",
                    5.,
                ),
                SiteKind::Resonator => game.say("Hold R    feed the structure your light", 3.),
                SiteKind::Ending if game.resonators >= 3 => {
                    game.say("Hold Shift    surrender your light", 4.)
                }
                _ => {}
            }
        }
        if d > 85. {
            continue;
        }
        match site.kind {
            SiteKind::AnchorAbility if game.pulse > 0.2 => {
                game.has_anchor = true;
                site.active = true;
                game.checkpoint = site.position;
                game.invulnerable = 5.;
                game.say(
                    "Lantern placement acquired\nE    set down a lantern. Stay still for a moment.\nIts steady light holds ribbed membranes open; pulses cannot.\nIt burns for 28 seconds. A new placement extinguishes the old one.\nOil extends the next placement to one minute.\n\nEnter    continue",
                    8.,
                );
                cues.0.push(Cue::Ability);
                game.mode = Mode::Discovery;
                game.pulse = 0.95;
                game.target = None;
                changed = true;
            }
            SiteKind::VeilAbility if game.pulse > 0.2 => {
                game.has_veil = true;
                site.active = true;
                game.checkpoint = site.position;
                game.invulnerable = 5.;
                game.say("Veil acquired\nHold Shift / X to extinguish and pass through woven curtains.\nReturn to the woven passage above Split Nerve.\n\nEnter    continue", 8.);
                cues.0.push(Cue::Ability);
                game.mode = Mode::Discovery;
                game.pulse = 0.95;
                game.target = None;
                changed = true;
            }
            SiteKind::Resonator => {
                charging = true;
                let nourished = if site.room == 16 {
                    crate::ritual::state(&game, site.position) == crate::ritual::FoldedState::Ready
                } else {
                    game.brightness > 0.35 || game.pulse > 0.2
                };
                if nourished && keys.pressed(KeyCode::KeyR) {
                    game.progress += dt / 3.5;
                } else {
                    game.progress = (game.progress - dt * 0.3).max(0.);
                }
                if game.progress >= 1. {
                    game.resonators += 1;
                    game.checkpoint = site.position;
                    game.invulnerable = 4.;
                    game.progress = 0.;
                    site.active = true;
                    game.pulse = 2.7;
                    let message = if game.resonators == 3 {
                        "3 / 3 structures awake\nThe passage at the Hollow is open. Return there."
                    } else {
                        "A light returns to the Hollow\nYour progress is saved."
                    };
                    game.say(message, 5.);
                    cues.0.push(Cue::Resonator);
                    changed = true;
                }
            }
            SiteKind::Secret if game.pulse > 0.2 => {
                game.secrets += 1;
                site.active = true;
                game.pulse = 2.;
                cues.0.push(Cue::Secret);
                game.say("A light follows you", 3.);
                changed = true;
            }
            SiteKind::Ending if game.resonators >= 3 => {
                charging = true;
                if game.brightness < 0.09 && game.pulse <= 0. {
                    game.progress += dt / 4.;
                } else {
                    game.progress = 0.;
                }
                if game.progress >= 1. {
                    game.mode = Mode::Ending;
                    game.ending_time = 0.;
                    site.active = true;
                    game.target = None;
                    cues.0.push(Cue::Ending);
                    changed = true;
                }
            }
            _ => {}
        }
    }
    if !charging {
        game.progress = 0.;
    }
    if changed {
        save::write_save(&game, &world);
    }
}
fn housekeeping(
    time: Res<Time>,
    world: Res<WorldMap>,
    mut game: ResMut<Game>,
    mut cues: ResMut<AudioCues>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if game.mode == Mode::Playing && windows.single().is_ok_and(|w| !w.focused) {
        game.mode = Mode::Paused;
        game.target = None;
    }
    let dt = time.delta_secs().min(0.05);
    if game.mode == Mode::Ending {
        game.ending_time += dt;
        game.elapsed += dt;
        return;
    }
    if game.mode != Mode::Playing || game.map_open {
        return;
    }
    game.elapsed += dt;
    game.pulse = (game.pulse - dt).max(0.);
    game.pulse_cooldown = (game.pulse_cooldown - dt).max(0.);
    game.anchor_charge = (game.anchor_charge - dt).max(0.);
    if game.anchor.is_some() {
        game.anchor_life = (game.anchor_life - dt).max(0.);
        if game.anchor_life <= 0. {
            game.anchor = None;
            cues.0.push(Cue::Scuff);
        }
    }
    if game.anchor_placing > 0. {
        game.anchor_placing = (game.anchor_placing - dt).max(0.);
        if game.anchor_placing <= 0. {
            game.anchor = Some(game.anchor_origin);
            game.anchor_shell = game.anchor;
            let used_oil = game.oil > 0;
            game.anchor_life = if used_oil {
                game.oil -= 1;
                60.
            } else {
                28.
            };
            if used_oil {
                save::write_save(&game, &world);
            }
            game.anchor_snuff = 0.;
            cues.0.push(Cue::Anchor);
        }
    }
    game.invulnerable = (game.invulnerable - dt).max(0.);
    game.brace = (game.brace - dt).max(0.);
    game.hurt_flash = (game.hurt_flash - dt * 2.2).max(0.);
    game.foot_echo = (game.foot_echo - dt * 1.8).max(0.);
    game.message_time = (game.message_time - dt).max(0.);
    game.room_name_time = (game.room_name_time - dt).max(0.);
    if game.brightness < 0.1 {
        game.dark_time += dt;
    } else {
        game.dark_time = 0.;
    }
    // Record milestones without clearing unrelated pickups or door guidance.
    if game.hint_stage == 0 && game.moved > 30. {
        game.hint_stage = 1;
    }
    if game.hint_stage < 2 && game.pulses > 0 {
        game.hint_stage = 2;
    }
    if game.hint_stage < 3 && game.room == 2 && game.dark_time > 1. {
        game.hint_stage = 3;
    }
    if game.hint_stage < 4 && game.room == 3 && game.message_time <= 0. {
        game.say("Wake three structures to open the passage here.\nTab    view your route, abilities and progress", 9.);
        game.hint_stage = 4;
    }
    let region = world.rooms[game.room].region as usize;
    if !game.region_seen[region]
        && game.hint_stage >= 4
        && game.message_time <= 0.
        && (region != 3 || game.room == 19)
    {
        game.region_seen[region] = true;
        match region {
            1 => game.say("Roots loosen in light. A lantern holds them apart.", 5.),
            2 => game.say(
                "Footsteps carry across the stone. Extinguish to tread softly.",
                5.,
            ),
            3 if game.room == 19 => game.say(
                "The shadow consumes light. Leave an anchor and move away.",
                5.,
            ),
            _ => {}
        }
    }
    game.save_clock += dt;
    if game.save_clock > 8. {
        game.save_clock = 0.;
        save::write_save(&game, &world);
    }
}

#[cfg(test)]
#[path = "validation.rs"]
mod validation;
