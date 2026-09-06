//! Headless tests exercise the real input, movement, mechanism, ecosystem and
//! progression schedule. They complement (and cannot replace) browser playtests.
use super::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

fn simulation(seed: u64) -> App {
    let world = WorldMap::generate(seed);
    let mut game = Game::new(seed, world.spawn, world.rooms.len());
    game.mode = Mode::Playing;
    // The ordinary save function deliberately ignores games not started through
    // the title. Tests must never write over someone's native development save.
    game.started = false;
    game.invulnerable = 1000.;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .insert_resource(world)
        .insert_resource(game)
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(AudioCues::default())
        .add_plugins(GameplayPlugin);
    app.update();
    let creatures: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Creature>>()
        .iter(app.world())
        .collect();
    for entity in creatures {
        app.world_mut().despawn(entity);
    }
    app
}

fn tick(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
    }
}

fn key(app: &mut App, code: KeyCode, pressed: bool) {
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    if pressed {
        keys.press(code);
    } else {
        keys.release(code);
    }
}

fn tap(app: &mut App, code: KeyCode) {
    key(app, code, true);
    tick(app, 1);
    key(app, code, false);
    tick(app, 1);
}

fn place_lantern(app: &mut App) {
    while app.world().resource::<Game>().anchor_charge > 0. {
        tick(app, 1);
    }
    tap(app, KeyCode::KeyE);
    assert!(app.world().resource::<Game>().anchor_placing > 0.);
    tick(app, 24);
    assert!(app.world().resource::<Game>().anchor.is_some());
}

fn pulse(app: &mut App) {
    while app.world().resource::<Game>().pulse_cooldown > 0. {
        tick(app, 1);
    }
    tap(app, KeyCode::Space);
    if app.world().resource::<Game>().mode == Mode::Discovery {
        assert!(app.world().resource::<Game>().message.contains("acquired"));
        tap(app, KeyCode::Enter);
    }
}

fn place_player(app: &mut App, p: Vec2) {
    let mut game = app.world_mut().resource_mut::<Game>();
    game.player = p;
    game.velocity = Vec2::ZERO;
    game.target = None;
}

fn walk(app: &mut App, target: Vec2) {
    app.world_mut().resource_mut::<Game>().target = Some(target);
    for _ in 0..350 {
        tick(app, 1);
        if app.world().resource::<Game>().player.distance(target) < 14. {
            return;
        }
    }
    let game = app.world().resource::<Game>();
    panic!(
        "Traversal stuck at {:?}, target {target:?}, room {}",
        game.player, game.room
    );
}

fn organism(app: &mut App, kind: CreatureKind, position: Vec2, room: usize) -> Entity {
    app.world_mut()
        .spawn(Creature {
            kind,
            home: position,
            position,
            velocity: Vec2::ZERO,
            alert: 0.,
            phase: 0.,
            gait: 0.,
            last_known: position,
            sense_age: 20.,
            notice: 0.,
            facing: 1.,
            room,
        })
        .id()
}

fn charge_resonator(app: &mut App, room: usize) {
    key(app, KeyCode::KeyR, true);
    if room == 16 {
        let center = app.world().resource::<WorldMap>().rooms[room].center;
        walk(app, center + Vec2::Y * 155.);
        place_lantern(app);
        key(app, KeyCode::ShiftLeft, true);
        walk(app, center);
        tick(app, 130);
    } else {
        key(app, KeyCode::ShiftLeft, false);
        tick(app, 90);
    }
    key(app, KeyCode::KeyR, false);
    assert!(
        app.world()
            .resource::<WorldMap>()
            .sites
            .iter()
            .find(|s| s.room == room && s.kind == SiteKind::Resonator)
            .unwrap()
            .active,
        "The resonator in room {room} was not nourished by its light rule"
    );
}

#[test]
fn darkness_alone_cannot_bypass_the_first_light_lesson() {
    let mut app = simulation(72419);
    let gate = app.world().resource::<WorldMap>().gates[0].clone();
    place_player(&mut app, gate.position - gate.normal * 90.);
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 60);
    assert!(!app.world().resource::<WorldMap>().gates[0].latched);
    assert!(!can_move(
        app.world().resource::<WorldMap>(),
        gate.position,
        false,
        true
    ));
    pulse(&mut app);
    assert!(app.world().resource::<WorldMap>().gates[0].latched);
    walk(&mut app, gate.position + gate.normal * 85.);
}

#[test]
fn anchor_requires_proximity_then_latches_after_a_real_crossing() {
    let mut app = simulation(20260906);
    let index = app
        .world()
        .resource::<WorldMap>()
        .gates
        .iter()
        .position(|g| g.kind == GateKind::Anchor)
        .unwrap();
    let gate = app.world().resource::<WorldMap>().gates[index].clone();
    place_player(&mut app, gate.position - gate.normal * 95.);
    {
        let mut game = app.world_mut().resource_mut::<Game>();
        game.has_anchor = true;
        game.anchor = Some(gate.position - gate.normal * 181.);
        game.anchor_life = 28.;
    }
    tick(&mut app, 30);
    assert!(app.world().resource::<WorldMap>().gates[index].open < 0.01);
    app.world_mut().resource_mut::<Game>().anchor = None;
    place_lantern(&mut app);
    tick(&mut app, 20);
    assert!(app.world().resource::<WorldMap>().gates[index].open > 0.84);
    assert!(!app.world().resource::<WorldMap>().gates[index].latched);
    walk(&mut app, gate.position + gate.normal * 85.);
    assert!(app.world().resource::<WorldMap>().gates[index].latched);
    tick(&mut app, 600);
    assert!(app.world().resource::<Game>().anchor.is_none());
    assert!(app.world().resource::<WorldMap>().gates[index].open > 0.98);
}

#[test]
fn veil_is_a_dark_traversal_verb_not_a_permanent_light_upgrade() {
    let mut app = simulation(913);
    let gate = app
        .world()
        .resource::<WorldMap>()
        .gates
        .iter()
        .find(|g| g.kind == GateKind::Veil)
        .unwrap()
        .clone();
    let world = app.world().resource::<WorldMap>();
    assert!(!can_move(world, gate.position, false, true));
    assert!(!can_move(world, gate.position, true, false));
    assert!(can_move(world, gate.position, true, true));
    app.world_mut().resource_mut::<Game>().has_veil = true;
    place_player(&mut app, gate.position - gate.normal * 90.);
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 20);
    walk(&mut app, gate.position + gate.normal * 85.);
}

#[test]
fn illumination_calls_listeners_and_darkness_releases_still_organisms() {
    let mut app = simulation(37);
    let world = app.world().resource::<WorldMap>();
    let center = world.rooms[5].center;
    let approach = world
        .passages
        .iter()
        .find(|p| p.a == 5 || p.b == 5)
        .unwrap();
    let neighbour = if approach.a == 5 {
        approach.points[1]
    } else {
        approach.points[approach.points.len() - 2]
    };
    let start = center + (neighbour - center).normalize() * 200.;
    assert!(world.field(start) > 38. && line_clear(world, center, start));
    place_player(&mut app, center);
    let listener = organism(&mut app, CreatureKind::Listener, start, 5);
    tick(&mut app, 30);
    assert!(app.world().get::<Creature>(listener).unwrap().alert > 0.6);
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 90);
    assert!(app.world().get::<Creature>(listener).unwrap().alert < 0.1);
    app.world_mut().despawn(listener);
    place_player(&mut app, center);
    key(&mut app, KeyCode::ShiftLeft, false);
    tick(&mut app, 20);
    let still = organism(&mut app, CreatureKind::Still, start, 5);
    tick(&mut app, 20);
    assert!(
        app.world()
            .get::<Creature>(still)
            .unwrap()
            .position
            .distance(start)
            < 1.
    );
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 65);
    assert!(
        app.world()
            .get::<Creature>(still)
            .unwrap()
            .position
            .distance(center)
            < 110.
    );
}

#[test]
fn an_anchor_holds_a_still_organism_while_the_player_remains_dark() {
    let mut app = simulation(81);
    let center = app.world().resource::<WorldMap>().rooms[5].center;
    place_player(&mut app, center);
    app.world_mut().resource_mut::<Game>().has_anchor = true;
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 20);
    place_lantern(&mut app);
    let world = app.world().resource::<WorldMap>();
    let start = (0..48)
        .map(|i| center + Vec2::from_angle(i as f32 * std::f32::consts::TAU / 48.) * 210.)
        .find(|p| (0..=30).all(|i| world.field(center.lerp(*p, i as f32 / 30.)) > 30.))
        .expect("an unobstructed approach to the Anchor");
    let still = organism(&mut app, CreatureKind::Still, start, 5);
    tick(&mut app, 40);
    let game = app.world().resource::<Game>();
    assert!(game.brightness < 0.1 && game.pulse == 0.);
    assert!(game.anchor.is_some());
    assert!(
        app.world()
            .get::<Creature>(still)
            .unwrap()
            .position
            .distance(start)
            < 1.,
        "A nearby anchor must illuminate the organism independently of the body"
    );
    tap(&mut app, KeyCode::KeyR);
    tick(&mut app, 85);
    assert!(app.world().resource::<Game>().anchor.is_none());
    assert!(
        app.world()
            .get::<Creature>(still)
            .unwrap()
            .position
            .distance(center)
            < 65.,
        "Recalling the anchor must release the organism again"
    );
}

#[test]
fn a_listener_cannot_follow_a_turn_it_did_not_see() {
    let mut app = simulation(418);
    let world = app.world().resource::<WorldMap>();
    // Use a known clear ungated throat; a random offset in a landmark may now
    // be separated by cover, which would test occlusion instead of dark pursuit.
    let passage = world
        .passages
        .iter()
        .find(|p| p.a == 2 && p.b == 3)
        .unwrap();
    let center = passage.points[2].lerp(passage.points[3], 0.5);
    assert!(line_clear(
        world,
        center - Vec2::X * 95.,
        center + Vec2::X * 60.
    ));
    place_player(&mut app, center + Vec2::X * 60.);
    let listener = organism(&mut app, CreatureKind::Listener, center - Vec2::X * 95., 14);
    app.world_mut().get_mut::<Creature>(listener).unwrap().alert = 0.8;
    tick(&mut app, 10);
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 10);
    assert!(app.world().resource::<Game>().brightness < 0.1);
    let creature = app.world().get::<Creature>(listener).unwrap();
    let last_direction = creature.velocity.normalize();
    let last_position = creature.position;
    assert!(creature.alert > 0.6);

    // Turn at a right angle through actual player input after extinguishing.
    key(&mut app, KeyCode::KeyW, true);
    tick(&mut app, 28);
    let creature = app.world().get::<Creature>(listener).unwrap();
    let direction_to_player =
        (app.world().resource::<Game>().player - creature.position).normalize();
    assert!(creature.velocity.normalize().dot(last_direction) > 0.999);
    assert!(
        (creature.position - last_position)
            .normalize()
            .dot(last_direction)
            > 0.999
    );
    assert!(
        direction_to_player.dot(last_direction) < 0.96,
        "The player must have turned far enough to distinguish searching from tracking"
    );
}

#[test]
fn a_long_browser_frame_cannot_tunnel_through_a_closed_gate() {
    let mut app = simulation(72419);
    let gate = app.world().resource::<WorldMap>().gates[0].clone();
    place_player(&mut app, gate.position - gate.normal * 90.);
    app.world_mut().resource_mut::<Game>().velocity = gate.normal * 182.;
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(3)));
    key(&mut app, KeyCode::KeyD, true);
    let before = app.world().resource::<Game>().player;
    tick(&mut app, 1);
    assert!(app.world().resource::<Game>().player.distance(before) <= 9.11);
    tick(&mut app, 30);
    let player = app.world().resource::<Game>().player;
    assert!(
        (player - gate.position).dot(gate.normal) <= -22.,
        "A frame stall must not turn normal movement speed into a gate bypass"
    );
    assert!(!app.world().resource::<WorldMap>().gates[0].latched);
}

#[test]
fn the_folded_resonator_rejects_direct_light_and_accepts_a_dark_approach_to_an_anchor() {
    let mut app = simulation(509);
    let center = app.world().resource::<WorldMap>().rooms[16].center;
    place_player(&mut app, center);
    app.world_mut().resource_mut::<Game>().has_anchor = true;
    for _ in 0..3 {
        pulse(&mut app);
        tick(&mut app, 80);
    }
    assert_eq!(app.world().resource::<Game>().resonators, 0);
    assert_eq!(app.world().resource::<Game>().progress, 0.);

    walk(&mut app, center + Vec2::Y * 155.);
    place_lantern(&mut app);
    walk(&mut app, center);
    tick(&mut app, 90);
    assert!(app.world().resource::<Game>().anchor.is_some());
    assert_eq!(
        app.world().resource::<Game>().resonators,
        0,
        "Even with a nearby anchor, the player's direct light must keep it folded"
    );
    key(&mut app, KeyCode::ShiftLeft, true);
    key(&mut app, KeyCode::KeyR, true);
    tick(&mut app, 120);
    assert_eq!(app.world().resource::<Game>().resonators, 1);
    assert!(
        app.world()
            .resource::<WorldMap>()
            .sites
            .iter()
            .find(|s| s.room == 16)
            .unwrap()
            .active
    );
}

#[test]
fn completion_requires_three_resonators_and_a_deliberate_final_extinction() {
    let mut app = simulation(17);
    let ending = app.world().resource::<WorldMap>().rooms[23].center;
    place_player(&mut app, ending);
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 120);
    assert_eq!(app.world().resource::<Game>().mode, Mode::Playing);
    key(&mut app, KeyCode::ShiftLeft, false);
    app.world_mut().resource_mut::<Game>().has_anchor = true;
    for room in [14, 16, 22] {
        let center = app.world().resource::<WorldMap>().rooms[room].center;
        place_player(&mut app, center);
        charge_resonator(&mut app, room);
    }
    assert_eq!(app.world().resource::<Game>().resonators, 3);
    place_player(&mut app, ending);
    tick(&mut app, 100);
    assert_eq!(app.world().resource::<Game>().mode, Mode::Playing);
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 120);
    assert_eq!(app.world().resource::<Game>().mode, Mode::Ending);
}

#[test]
fn death_preserves_abilities_activated_sites_and_shortcuts() {
    let mut app = simulation(312);
    let center = app.world().resource::<WorldMap>().rooms[14].center + Vec2::X * 120.;
    place_player(&mut app, center);
    {
        let mut game = app.world_mut().resource_mut::<Game>();
        game.has_anchor = true;
        game.has_veil = true;
        game.resonators = 1;
        game.invulnerable = 0.;
    }
    {
        let mut world = app.world_mut().resource_mut::<WorldMap>();
        world.gates[0].latched = true;
        world
            .sites
            .iter_mut()
            .find(|s| s.room == 14)
            .unwrap()
            .active = true;
    }
    let listener = organism(&mut app, CreatureKind::Listener, center, 14);
    app.world_mut().get_mut::<Creature>(listener).unwrap().alert = 1.;
    tick(&mut app, 70);
    let game = app.world().resource::<Game>();
    assert_eq!(game.mode, Mode::Dead);
    assert!(game.has_anchor && game.has_veil);
    assert_eq!(game.resonators, 1);
    assert!(app.world().resource::<WorldMap>().gates[0].latched);
    assert!(
        app.world()
            .resource::<WorldMap>()
            .sites
            .iter()
            .find(|s| s.room == 14)
            .unwrap()
            .active
    );
}

#[test]
fn the_authored_journey_is_walkable_using_the_actual_game_systems() {
    for seed in [0, 72419, 20260906] {
        let mut app = simulation(seed);
        let journey = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 3, 9, 10, 9, 3, 4, 5, 16, 15, 21, 14, 13, 12, 22, 19, 18,
            17, 9, 3, 23,
        ];
        for leg in journey.windows(2) {
            let (from, to) = (leg[0], leg[1]);
            let world = app.world().resource::<WorldMap>();
            let a = world.rooms[from].center;
            let b = world.rooms[to].center;
            let passage = world
                .passages
                .iter()
                .find(|p| (p.a == from && p.b == to) || (p.b == from && p.a == to))
                .unwrap();
            let gate = passage.gate.map(|index| world.gates[index].clone());
            let mut waypoints = passage.points.clone();
            if passage.a != from {
                waypoints.reverse();
            }
            for point in &waypoints[1..3] {
                walk(&mut app, *point);
            }
            if let Some(gate) = gate {
                let normal = (b - a).normalize();
                walk(&mut app, gate.position - normal * 95.);
                match gate.kind {
                    GateKind::Light => pulse(&mut app),
                    GateKind::Shade | GateKind::Veil => {
                        key(&mut app, KeyCode::ShiftLeft, true);
                        tick(&mut app, 65);
                    }
                    GateKind::Anchor => {
                        if !gate.latched {
                            place_lantern(&mut app);
                            tick(&mut app, 20);
                        }
                    }
                    GateKind::Final => tick(&mut app, 20),
                }
            }
            for point in &waypoints[3..] {
                walk(&mut app, *point);
            }
            if matches!(to, 6 | 10) {
                pulse(&mut app);
            }
            if matches!(to, 14 | 16 | 22) {
                charge_resonator(&mut app, to);
            }
        }
        assert!(app.world().resource::<Game>().has_anchor);
        assert!(app.world().resource::<Game>().has_veil);
        assert_eq!(app.world().resource::<Game>().resonators, 3);
        key(&mut app, KeyCode::ShiftLeft, true);
        tick(&mut app, 120);
        assert_eq!(
            app.world().resource::<Game>().mode,
            Mode::Ending,
            "seed {seed}"
        );
    }
}

#[test]
fn creatures_arrive_without_reversing_repeatedly_at_a_stationary_light() {
    let mut app = simulation(37);
    let center = app.world().resource::<WorldMap>().rooms[5].center;
    place_player(&mut app, center);
    let listener = organism(&mut app, CreatureKind::Listener, center + Vec2::X * 140., 5);
    app.world_mut().get_mut::<Creature>(listener).unwrap().alert = 0.8;
    let mut previous = 140.;
    for _ in 0..240 {
        tick(&mut app, 1);
        let creature = app.world().get::<Creature>(listener).unwrap();
        let distance = creature.position.distance(center);
        assert!(
            distance <= previous + 0.02,
            "Approach oscillated around its destination"
        );
        previous = distance;
    }
    let creature = app.world().get::<Creature>(listener).unwrap();
    assert!(creature.position.distance(center) < 1.);
    assert!(creature.velocity.length() < 1.);
}

#[test]
fn solid_clutter_blocks_movement_and_light_but_keeps_the_main_paths_clear() {
    for seed in 0..32 {
        let world = WorldMap::generate(seed);
        let mut count = 0;
        for room in &world.rooms {
            assert!(world.field(world.creature_spawn(room.id)) > 38.);
            for rock in &room.formations {
                let center = (rock.a + rock.b) * 0.5;
                assert!(!can_move(&world, center, true, true));
                let axis = (rock.b - rock.a).normalize();
                let across = Vec2::new(-axis.y, axis.x) * (rock.radius + 18.);
                assert!(!line_clear(&world, center - across, center + across));
                count += 1;
            }
        }
        assert!(
            count > world.rooms.len(),
            "World lacked meaningful physical clutter"
        );
        for passage in &world.passages {
            for w in passage.points.windows(2) {
                for step in 0..=32 {
                    assert!(world.field(w[0].lerp(w[1], step as f32 / 32.)) > 36.);
                }
            }
        }
    }
}

#[test]
fn footfalls_follow_ground_travel_and_stop_at_a_wall() {
    let mut app = simulation(72419);
    tick(&mut app, 60);
    assert!(!app.world().resource::<AudioCues>().0.contains(&Cue::Step));
    let gate = app.world().resource::<WorldMap>().gates[0].clone();
    place_player(&mut app, gate.position - gate.normal * 180.);
    key(&mut app, KeyCode::KeyD, true);
    tick(&mut app, 100);
    let game = app.world().resource::<Game>();
    let count = app
        .world()
        .resource::<AudioCues>()
        .0
        .iter()
        .filter(|cue| **cue == Cue::Step)
        .count();
    assert!(count >= 2);
    assert_eq!(count, (game.moved / 38.).floor() as usize);
    app.world_mut().resource_mut::<AudioCues>().0.clear();
    let before = app.world().resource::<Game>().moved;
    tick(&mut app, 100);
    assert!((app.world().resource::<Game>().moved - before).abs() < 0.01);
    assert!(!app.world().resource::<AudioCues>().0.contains(&Cue::Step));
    key(&mut app, KeyCode::KeyD, false);
    tick(&mut app, 40);
    assert!(app.world().resource::<Game>().velocity.length() < 0.01);
}

#[test]
fn a_click_destination_survives_a_blocked_membrane_and_resumes_when_it_opens() {
    let mut app = simulation(72419);
    let gate = app.world().resource::<WorldMap>().gates[0].clone();
    place_player(&mut app, gate.position - gate.normal * 80.);
    let target = gate.position + gate.normal * 100.;
    app.world_mut().resource_mut::<Game>().target = Some(target);
    tick(&mut app, 70);
    assert_eq!(app.world().resource::<Game>().target, Some(target));
    assert!(app.world().resource::<Game>().ground_speed < 0.1);
    assert!((app.world().resource::<Game>().player - gate.position).dot(gate.normal) < -21.);
    pulse(&mut app);
    tick(&mut app, 60);
    assert!(app.world().resource::<Game>().player.distance(target) < 14.);
}

#[test]
fn held_movement_continues_after_a_gate_opens_without_repressing_the_key() {
    let mut app = simulation(72419);
    let gate = app.world().resource::<WorldMap>().gates[0].clone();
    place_player(&mut app, gate.position - gate.normal * 80.);
    key(&mut app, KeyCode::KeyD, true);
    tick(&mut app, 70);
    let before = app.world().resource::<Game>().player;
    pulse(&mut app);
    tick(&mut app, 35);
    assert!(app.world().resource::<Game>().player.x > before.x + 80.);
    assert!(
        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::KeyD)
    );
}

#[test]
fn a_mouse_click_shorter_than_one_render_frame_is_not_dropped() {
    let mut app = simulation(72419);
    let mut window = Window {
        resolution: (1280, 800).into(),
        ..default()
    };
    window.set_cursor_position(Some(Vec2::new(1000., 400.)));
    app.world_mut().spawn((window, PrimaryWindow));
    let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    mouse.press(MouseButton::Left);
    mouse.release(MouseButton::Left);
    tick(&mut app, 1);
    let target = app
        .world()
        .resource::<Game>()
        .target
        .expect("A quick click must establish a destination");
    let before = app.world().resource::<Game>().player;
    tick(&mut app, 25);
    assert!(app.world().resource::<Game>().player.distance(target) < before.distance(target) - 40.);
}

#[test]
fn return_passage_contact_can_slide_up_and_left() {
    let mut app = simulation(72419);
    app.insert_resource(WorldMap::legacy(72419));
    place_player(&mut app, Vec2::new(1009., -27.));
    app.world_mut().resource_mut::<Game>().target = Some(Vec2::new(890., 18.));
    tick(&mut app, 40);
    let g = app.world().resource::<Game>();
    assert!(
        g.player.x < 990. && g.player.y > 5.,
        "contact stuck at {:?}",
        g.player
    );
}

#[test]
fn a_closed_membrane_blocks_detection_until_opened() {
    let mut app = simulation(72419);
    let gate = app.world().resource::<WorldMap>().gates[0].clone();
    let a = gate.position - gate.normal * 65.;
    let b = gate.position + gate.normal * 65.;
    place_player(&mut app, a);
    let listener = organism(&mut app, CreatureKind::Listener, b, 4);
    tick(&mut app, 10);
    assert!(!line_clear(app.world().resource::<WorldMap>(), a, b));
    assert_eq!(app.world().get::<Creature>(listener).unwrap().notice, 0.);
    app.world_mut().resource_mut::<WorldMap>().gates[0].latched = true;
    tick(&mut app, 10);
    assert!(line_clear(app.world().resource::<WorldMap>(), a, b));
    assert!(app.world().get::<Creature>(listener).unwrap().notice > 0.);
}

#[test]
fn first_contact_gives_time_to_escape_and_recover() {
    let mut app = simulation(72419);
    let p = app.world().resource::<WorldMap>().rooms[19].center;
    place_player(&mut app, p);
    app.world_mut().resource_mut::<Game>().invulnerable = 0.;
    organism(&mut app, CreatureKind::Listener, p, 19);
    tick(&mut app, 1);
    let game = app.world().resource::<Game>();
    assert_eq!(game.mode, Mode::Playing);
    assert!(game.invulnerable > 1. && game.brace > 6.);
    assert!(game.hurt_flash > 0.8);
    assert!(app.world().resource::<AudioCues>().0.contains(&Cue::Hurt));
    key(&mut app, KeyCode::KeyD, true);
    tick(&mut app, 65);
    assert_eq!(app.world().resource::<Game>().mode, Mode::Playing);
    assert!(app.world().resource::<Game>().player.distance(p) > 90.);
}

#[test]
fn stopping_plants_the_feet_without_idle_footsteps() {
    let mut app = simulation(72419);
    key(&mut app, KeyCode::KeyD, true);
    tick(&mut app, 22);
    key(&mut app, KeyCode::KeyD, false);
    tick(&mut app, 40);
    let gait = app.world().resource::<Game>().gait;
    assert!((gait / 38. - (gait / 38.).round()).abs() < 0.001);
    app.world_mut().resource_mut::<AudioCues>().0.clear();
    tick(&mut app, 40);
    assert!(!app.world().resource::<AudioCues>().0.contains(&Cue::Step));
}

#[test]
fn a_completed_resonator_provides_a_safe_checkpoint() {
    let mut app = simulation(72419);
    let p = app.world().resource::<WorldMap>().rooms[14].center;
    place_player(&mut app, p);
    charge_resonator(&mut app, 14);
    let game = app.world().resource::<Game>();
    let site = app
        .world()
        .resource::<WorldMap>()
        .sites
        .iter()
        .find(|s| s.room == 14)
        .unwrap();
    assert_eq!(game.checkpoint, site.position);
    assert!(site.active);
    app.world_mut().resource_mut::<Game>().invulnerable = 0.;
    organism(&mut app, CreatureKind::Listener, p, 14);
    tick(&mut app, 100);
    assert_eq!(app.world().resource::<Game>().mode, Mode::Playing);
}

#[test]
fn shadow_consumes_placed_light_and_recedes_after_the_deep_structure() {
    let mut app = simulation(72419);
    let home = app.world().resource::<WorldMap>().rooms[19].center;
    place_player(&mut app, home - Vec2::Y * 250.);
    let entity = organism(&mut app, CreatureKind::Leviathan, home, 19);
    let branch = crate::shadow::branches(app.world().get::<Creature>(entity).unwrap())[0];
    app.world_mut().resource_mut::<Game>().anchor = Some(branch.xy());
    app.world_mut().resource_mut::<Game>().anchor_life = 28.;
    tick(&mut app, 1);
    assert!(app.world().resource::<Game>().anchor.is_some());
    tick(&mut app, 32);
    assert!(app.world().resource::<Game>().anchor.is_none());
    app.world_mut()
        .resource_mut::<WorldMap>()
        .sites
        .iter_mut()
        .find(|s| s.room == 22)
        .unwrap()
        .active = true;
    tick(&mut app, 80);
    assert!(app.world().get::<Creature>(entity).unwrap().position.y > home.y + 50.);
}

#[test]
fn stone_footfalls_are_louder_signals_than_extinguished_steps() {
    let mut app = simulation(72419);
    let p = app.world().resource::<WorldMap>().rooms[12].center;
    place_player(&mut app, p);
    key(&mut app, KeyCode::KeyW, true);
    tick(&mut app, 18);
    assert!(app.world().resource::<Game>().foot_echo > 0.3);
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 25);
    assert!(app.world().resource::<Game>().foot_echo <= 0.25);
}

#[test]
fn acquisition_waits_for_acknowledgement_with_a_direct_mechanical_description() {
    let mut app = simulation(72419);
    let p = app.world().resource::<WorldMap>().rooms[6].center;
    place_player(&mut app, p);
    tap(&mut app, KeyCode::Space);
    assert_eq!(app.world().resource::<Game>().mode, Mode::Discovery);
    assert!(
        app.world()
            .resource::<Game>()
            .message
            .contains("Lantern placement acquired")
    );
    let before = app.world().resource::<Game>().player;
    key(&mut app, KeyCode::KeyD, true);
    tick(&mut app, 30);
    assert_eq!(app.world().resource::<Game>().player, before);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(app.world().resource::<Game>().mode, Mode::Playing);
}

#[test]
fn roots_pull_into_the_wall_and_release_the_same_collision_geometry() {
    let mut app = simulation(72419);
    let roots = app.world().resource::<crate::roots::Rootwork>();
    let world = app.world().resource::<WorldMap>();
    let p = roots
        .patches
        .iter()
        .flat_map(|r| r.visible_segments())
        .map(|s| s.a.lerp(s.b, 0.5))
        .find(|p| world.field(*p) > 30. && roots.dragging(*p))
        .expect("navigable branching roots");
    place_player(&mut app, p);
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 80);
    assert!(app.world().resource::<crate::roots::Rootwork>().dragging(p));
    app.world_mut().resource_mut::<Game>().has_anchor = true;
    place_lantern(&mut app);
    tick(&mut app, 30);
    assert!(app.world().resource::<Game>().brightness < 0.1);
    assert!(!app.world().resource::<crate::roots::Rootwork>().dragging(p));
    tap(&mut app, KeyCode::KeyR);
    tick(&mut app, 200);
    assert!(app.world().resource::<crate::roots::Rootwork>().dragging(p));
}

#[test]
fn lantern_placement_can_be_interrupted_replaces_the_old_one_and_burns_out() {
    let mut app = simulation(72419);
    app.world_mut().resource_mut::<Game>().has_anchor = true;
    tap(&mut app, KeyCode::KeyE);
    tick(&mut app, 10);
    assert!(app.world().resource::<Game>().anchor.is_none());
    key(&mut app, KeyCode::KeyD, true);
    tick(&mut app, 5);
    key(&mut app, KeyCode::KeyD, false);
    tick(&mut app, 30);
    assert!(app.world().resource::<Game>().anchor.is_none());
    place_lantern(&mut app);
    let first = app.world().resource::<Game>().anchor.unwrap();
    assert!(app.world().resource::<Game>().anchor_life > 27.);
    walk(&mut app, first + Vec2::X * 90.);
    while app.world().resource::<Game>().anchor_charge > 0. {
        tick(&mut app, 1);
    }
    tap(&mut app, KeyCode::KeyE);
    assert_eq!(app.world().resource::<Game>().anchor, Some(first));
    tick(&mut app, 24);
    assert!(
        app.world()
            .resource::<Game>()
            .anchor
            .unwrap()
            .distance(first)
            > 70.
    );
    tick(&mut app, 560);
    assert!(app.world().resource::<Game>().anchor.is_none());
}

#[test]
fn oil_is_a_single_use_find_and_only_extends_a_completed_placement() {
    let mut app = simulation(72419);
    let p = crate::exploration::finds(app.world().resource::<WorldMap>())
        .into_iter()
        .find(|f| f.kind == crate::exploration::FindKind::Oil)
        .unwrap()
        .position;
    place_player(&mut app, p);
    tap(&mut app, KeyCode::KeyR);
    assert_eq!(app.world().resource::<Game>().oil, 1);
    tap(&mut app, KeyCode::KeyR);
    assert_eq!(app.world().resource::<Game>().oil, 1);
    app.world_mut().resource_mut::<Game>().has_anchor = true;
    tap(&mut app, KeyCode::KeyE);
    assert_eq!(app.world().resource::<Game>().oil, 1);
    tick(&mut app, 24);
    let game = app.world().resource::<Game>();
    assert_eq!(game.oil, 0);
    assert!(game.anchor_life > 59.);
}

#[test]
fn a_bell_can_be_found_thrown_heard_and_retrieved_without_light() {
    let mut app = simulation(72419);
    let p = crate::exploration::finds(app.world().resource::<WorldMap>())[0].position;
    place_player(&mut app, p);
    tap(&mut app, KeyCode::KeyR);
    assert!(app.world().resource::<Game>().bell_found);
    assert_eq!(app.world().resource::<Game>().mode, Mode::Discovery);
    tap(&mut app, KeyCode::Enter);
    let world = app.world().resource::<WorldMap>();
    let center = world.rooms[5].center;
    let dir = (0..32)
        .map(|i| Vec2::from_angle(i as f32 * std::f32::consts::TAU / 32.))
        .find(|d| (0..28).all(|i| world.field(center + *d * i as f32 * 7.) > 30.))
        .unwrap();
    place_player(&mut app, center);
    app.world_mut().resource_mut::<Game>().facing = dir;
    key(&mut app, KeyCode::ShiftLeft, true);
    tick(&mut app, 80);
    let entity = organism(&mut app, CreatureKind::Listener, center + dir * 180., 5);
    tap(&mut app, KeyCode::KeyQ);
    let bell = app.world().resource::<Game>().bell_out.unwrap();
    assert!(bell.distance(center) > 100.);
    tick(&mut app, 20);
    assert!(
        app.world()
            .get::<Creature>(entity)
            .unwrap()
            .last_known
            .distance(bell)
            < 1.
    );
    tap(&mut app, KeyCode::KeyQ);
    assert_eq!(app.world().resource::<Game>().bell_out, Some(bell));
    walk(&mut app, bell);
    tap(&mut app, KeyCode::KeyR);
    assert!(app.world().resource::<Game>().bell_out.is_none());
    assert!(app.world().resource::<Game>().bell_found);
}

#[test]
fn standing_in_light_does_not_automatically_complete_a_structure() {
    let mut app = simulation(72419);
    let p = app.world().resource::<WorldMap>().rooms[14].center;
    place_player(&mut app, p);
    tick(&mut app, 140);
    assert_eq!(app.world().resource::<Game>().resonators, 0);
    charge_resonator(&mut app, 14);
}

#[test]
fn route_stones_mark_required_structures_and_finds_have_clear_approaches() {
    for seed in 0..128 {
        let world = WorldMap::generate(seed);
        for f in crate::exploration::finds(&world) {
            assert!(world.field(f.position) > 25., "seed {seed}, find {}", f.id);
            let center = world.rooms[world.room_at(f.position)].center;
            assert!(line_clear(&world, center, f.position));
        }
    }
    let mut app = simulation(72419);
    let p = crate::exploration::finds(app.world().resource::<WorldMap>())[1].position;
    place_player(&mut app, p);
    tap(&mut app, KeyCode::KeyR);
    assert_eq!(app.world().resource::<Game>().charted, 1);
}

#[test]
fn a_listener_takes_time_to_extinguish_a_placed_lantern() {
    let mut app = simulation(72419);
    let center = app.world().resource::<WorldMap>().rooms[5].center;
    place_player(&mut app, center + Vec2::X * 140.);
    key(&mut app, KeyCode::ShiftLeft, true);
    {
        let mut game = app.world_mut().resource_mut::<Game>();
        game.anchor = Some(center);
        game.anchor_life = 28.;
    }
    organism(&mut app, CreatureKind::Listener, center, 5);
    tick(&mut app, 20);
    assert!(app.world().resource::<Game>().anchor.is_some());
    assert!(app.world().resource::<Game>().anchor_snuff > 0.8);
    tick(&mut app, 12);
    assert!(app.world().resource::<Game>().anchor.is_none());
}
