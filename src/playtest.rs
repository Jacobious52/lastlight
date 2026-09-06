//! Deliberate, opt-in development starts. Never enabled in the release build.
use crate::{model::*, world::*};
pub fn apply(game: &mut Game, world: &mut WorldMap) {
    #[cfg(target_arch = "wasm32")]
    let selection = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .and_then(|s| {
            s.trim_start_matches('?')
                .split('&')
                .find_map(|p| p.strip_prefix("playtest=").map(str::to_owned))
        });
    #[cfg(not(target_arch = "wasm32"))]
    let selection = std::env::args().find_map(|p| p.strip_prefix("--playtest=").map(str::to_owned));
    let Some(selection) = selection else {
        return;
    };
    let ritual = selection == "ritual";
    let survey = selection == "survey";
    let ending = selection == "ending";
    let opening = selection == "opening";
    let anchor_lesson = selection == "anchor";
    let veil_lesson = selection == "veil";
    let gate_preview = selection
        .strip_prefix("gate-")
        .and_then(|n| n.parse::<usize>().ok());
    let room = if ritual {
        16
    } else if anchor_lesson {
        6
    } else if veil_lesson {
        10
    } else if ending {
        23
    } else {
        selection
            .parse::<usize>()
            .unwrap_or(0)
            .min(world.rooms.len() - 1)
    };
    *game = Game::new(world.seed, world.rooms[room].center, world.rooms.len());
    game.test_mode = true;
    game.bell_found = ritual;
    if survey {
        game.visited.fill(true);
        game.map_open = true;
    }
    game.has_anchor = !opening && !anchor_lesson;
    game.has_veil = !opening && !anchor_lesson && !veil_lesson;
    game.hint_stage = if opening { 0 } else { 4 };
    game.visited[room] = true;
    game.room = room;
    game.checkpoint = game.player;
    game.invulnerable = 4.;
    if opening {
        for gate in &mut world.gates {
            gate.latched = false;
            gate.open = 0.;
        }
        for site in &mut world.sites {
            site.active = false;
        }
        game.player = world.spawn;
        game.camera = world.spawn;
        game.checkpoint = world.spawn;
        return;
    }
    for site in &mut world.sites {
        site.active = (site.kind == SiteKind::AnchorAbility && game.has_anchor)
            || (site.kind == SiteKind::VeilAbility && game.has_veil)
            || ending && site.kind == SiteKind::Resonator;
    }
    if ending {
        game.resonators = 3;
    }
    for gate in &mut world.gates {
        gate.latched = (gate.kind == GateKind::Light
            || gate.kind == GateKind::Anchor && !anchor_lesson)
            || ending && gate.kind == GateKind::Final;
        gate.open = if gate.latched { 1. } else { 0. };
    }
    if let Some(index) = gate_preview
        && let Some(gate) = world.gates.get_mut(index)
    {
        gate.latched = false;
        gate.open = 0.;
        game.player = gate.position - gate.normal * 110.;
        game.camera = game.player;
        game.checkpoint = game.player;
        game.room = world.room_at(game.player);
        game.visited[game.room] = true;
    }
}
