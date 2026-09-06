use crate::{model::*, world::*};
use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};

const SAVE_VERSION: u32 = 1;
#[cfg(any(target_arch = "wasm32", test))]
const STORAGE_KEY: &str = "lastlight-save-v1";
const DEFAULT_SEED: u64 = 72419;
const MAX_SAVE_BYTES: usize = 64 * 1024;
#[cfg(not(target_arch = "wasm32"))]
const SAVE_PATH: &str = ".lastlight-save.json";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Save {
    version: u32,
    seed: u64,
    #[serde(default)]
    world_revision: u32,
    checkpoint: [f32; 2],
    anchor: bool,
    veil: bool,
    resonators: u32,
    secrets: u32,
    visited: Vec<bool>,
    sites: Vec<bool>,
    gates: Vec<bool>,
    elapsed: f32,
    deaths: u32,
    pulses: u32,
    #[serde(default)]
    oil: u32,
    #[serde(default)]
    collected: u32,
    #[serde(default)]
    charted: u32,
    #[serde(default)]
    bell_found: bool,
}

impl Save {
    fn snapshot(game: &Game, world: &WorldMap) -> Self {
        Self {
            version: SAVE_VERSION,
            seed: world.seed,
            world_revision: 2,
            checkpoint: game.checkpoint.to_array(),
            anchor: game.has_anchor,
            veil: game.has_veil,
            resonators: game.resonators,
            secrets: game.secrets,
            visited: game.visited.clone(),
            sites: world.sites.iter().map(|site| site.active).collect(),
            gates: world.gates.iter().map(|gate| gate.latched).collect(),
            elapsed: game.elapsed,
            deaths: game.deaths,
            pulses: game.pulses,
            oil: game.oil,
            collected: game.collected,
            charted: game.charted,
            bell_found: game.bell_found,
        }
    }

    fn valid_scalars(&self) -> bool {
        self.version == SAVE_VERSION
            && matches!(self.world_revision, 0 | 2)
            && self.elapsed.is_finite()
            && self.elapsed >= 0.
            && self.checkpoint.iter().all(|value| value.is_finite())
            && !self.visited.is_empty()
            && self.visited.len() <= 256
            && !self.sites.is_empty()
            && self.sites.len() <= 256
            && self.gates.len() <= 256
            && self.resonators <= self.sites.len() as u32
            && self.secrets <= self.sites.len() as u32
            && self.deaths < u32::MAX
            && self.pulses < u32::MAX
            && self.oil <= 2
            && self.collected < (1 << 11)
            && self.charted <= 7
            && self.bell_found == (self.collected & 1 != 0)
    }

    /// Validate before touching either resource: a partial restore can leave a
    /// seemingly new game with abilities, barriers, or collectibles out of sync.
    fn valid_for(&self, world: &WorldMap) -> bool {
        if !self.valid_scalars()
            || self.seed != world.seed
            || self.visited.len() != world.rooms.len()
            || self.sites.len() != world.sites.len()
            || self.gates.len() != world.gates.len()
        {
            return false;
        }
        let active_count = |kind| {
            world
                .sites
                .iter()
                .zip(&self.sites)
                .filter(|(site, active)| site.kind == kind && **active)
                .count() as u32
        };
        if self.anchor != (active_count(SiteKind::AnchorAbility) > 0)
            || self.veil != (active_count(SiteKind::VeilAbility) > 0)
            || (self.veil && !self.anchor)
            || self.resonators != active_count(SiteKind::Resonator)
            || self.secrets != active_count(SiteKind::Secret)
        {
            return false;
        }
        let required_resonators = world
            .sites
            .iter()
            .filter(|site| site.kind == SiteKind::Resonator)
            .count() as u32;
        if active_count(SiteKind::Ending) > 0
            && (!self.anchor || !self.veil || self.resonators != required_resonators)
        {
            return false;
        }
        // Only light-fed and anchored membranes become permanent shortcuts.
        // Other membranes must still react to illumination after a reload.
        if world.gates.iter().zip(&self.gates).any(|(gate, latched)| {
            *latched
                && match gate.kind {
                    GateKind::Light => false,
                    GateKind::Anchor => !self.anchor,
                    GateKind::Shade | GateKind::Veil | GateKind::Final => true,
                }
        }) {
            return false;
        }
        let checkpoint = Vec2::from_array(self.checkpoint);
        if world.field(checkpoint) < 15. {
            return false;
        }
        // The save stores an activated sanctuary, ability or resonator, not the
        // live player position. An arbitrary point can strand a restored player
        // behind a transient gate or inside a newly sleeping organism.
        checkpoint.distance_squared(world.spawn) < 1.
            || world.sites.iter().zip(&self.sites).any(|(site, active)| {
                *active
                    && matches!(
                        site.kind,
                        SiteKind::Sanctuary
                            | SiteKind::AnchorAbility
                            | SiteKind::VeilAbility
                            | SiteKind::Resonator
                    )
                    && checkpoint.distance_squared(site.position) < 1.
            })
    }
}

fn decode(text: &str) -> Option<Save> {
    if text.len() > MAX_SAVE_BYTES {
        return None;
    }
    serde_json::from_str::<Save>(text)
        .ok()
        .filter(Save::valid_scalars)
}

pub fn load() -> Option<Save> {
    #[cfg(target_arch = "wasm32")]
    let text = web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(STORAGE_KEY)
        .ok()??;
    #[cfg(not(target_arch = "wasm32"))]
    let text = std::fs::read_to_string(SAVE_PATH).ok()?;
    decode(&text)
}

#[cfg(any(target_arch = "wasm32", test))]
fn query_seed(search: &str) -> Option<u64> {
    search
        .trim_start_matches('?')
        .split('&')
        .filter_map(|pair| pair.strip_prefix("seed="))
        .find_map(|value| value.parse().ok())
}

pub fn seed(save: Option<&Save>) -> u64 {
    #[cfg(target_arch = "wasm32")]
    if let Some(explicit) = web_sys::window()
        .and_then(|window| window.location().search().ok())
        .and_then(|search| query_seed(&search))
    {
        return explicit;
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(explicit) = std::env::args()
        .filter_map(|argument| argument.strip_prefix("--seed=").map(str::to_owned))
        .find_map(|value| value.parse().ok())
    {
        return explicit;
    }
    save.filter(|save| save.valid_scalars())
        .map(|save| save.seed)
        .unwrap_or(DEFAULT_SEED)
}

pub fn restore(save: Option<Save>, game: &mut Game, world: &mut WorldMap) {
    let save = save.and_then(|mut saved| {
        if saved.world_revision == 0 {
            let old = WorldMap::legacy(saved.seed);
            if !saved.valid_for(&old) || saved.seed != world.seed {
                return None;
            }
            let checkpoint = Vec2::from_array(saved.checkpoint);
            saved.checkpoint = if checkpoint.distance_squared(old.spawn) < 1. {
                world.spawn.to_array()
            } else {
                let index = old
                    .sites
                    .iter()
                    .position(|s| s.position.distance_squared(checkpoint) < 1.)?;
                world.sites[index].position.to_array()
            };
            let mut gates = vec![false; world.gates.len()];
            for passage in &old.passages {
                if passage.gate.is_some_and(|i| saved.gates[i])
                    && let Some(index) = world
                        .passages
                        .iter()
                        .find(|p| p.a == passage.a && p.b == passage.b)
                        .and_then(|p| p.gate)
                {
                    gates[index] = true;
                }
            }
            saved.gates = gates;
            saved.world_revision = 2;
        }
        Some(saved)
    });
    let Some(saved) = save.filter(|saved| saved.valid_for(world)) else {
        return;
    };
    let checkpoint = Vec2::from_array(saved.checkpoint);
    game.player = checkpoint;
    game.camera = checkpoint;
    game.checkpoint = checkpoint;
    game.has_anchor = saved.anchor;
    game.has_veil = saved.veil;
    game.resonators = saved.resonators;
    game.secrets = saved.secrets;
    game.visited = saved.visited;
    game.elapsed = saved.elapsed;
    game.deaths = saved.deaths;
    game.pulses = saved.pulses;
    game.oil = saved.oil;
    game.collected = saved.collected;
    game.charted = saved.charted;
    game.bell_found = saved.bell_found;
    game.started = true;
    game.room = world.room_at(checkpoint);
    // A save during the guarded opening must still teach darkness and memory.
    game.hint_stage = if game.visited.get(3).copied().unwrap_or(false) {
        4
    } else if game.pulses > 0 {
        2
    } else {
        0
    };
    for (site, active) in world.sites.iter_mut().zip(saved.sites) {
        site.active = active;
    }
    for (gate, latched) in world.gates.iter_mut().zip(saved.gates) {
        gate.latched = latched;
        gate.open = if latched { 1. } else { 0. };
    }
    // Keep Ending.active: the title's Continue action uses it to replay the
    // conclusion. Do not silently turn a completed game into an unfinished one.
}

pub fn write_save(game: &Game, world: &WorldMap) {
    if !game.started || game.test_mode {
        return;
    }
    let saved = Save::snapshot(game, world);
    if !saved.valid_for(world) {
        return;
    }
    let Ok(text) = serde_json::to_string(&saved) else {
        return;
    };
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        // One intentional current-run slot. Merely visiting ?seed=another never
        // writes: a different run replaces it only after the player begins.
        let _ = storage.set_item(STORAGE_KEY, &text);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = write_atomic(&text);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn write_atomic(text: &str) -> std::io::Result<()> {
    use std::io::Write;
    // A sibling guarantees the final rename stays on the same filesystem. A
    // failed write/sync leaves the previous good save intact.
    let temporary = format!("{SAVE_PATH}.tmp");
    let mut file = std::fs::File::create(&temporary)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&temporary, SAVE_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh(seed: u64) -> (Game, WorldMap) {
        let world = WorldMap::generate(seed);
        let game = Game::new(seed, world.spawn, world.rooms.len());
        (game, world)
    }

    fn finished(seed: u64) -> (Game, WorldMap) {
        let (mut game, mut world) = fresh(seed);
        game.started = true;
        game.has_anchor = true;
        game.has_veil = true;
        game.elapsed = 1427.75;
        game.deaths = 4;
        game.pulses = 93;
        game.visited.fill(true);
        for site in &mut world.sites {
            site.active = true;
        }
        game.resonators = world
            .sites
            .iter()
            .filter(|site| site.kind == SiteKind::Resonator)
            .count() as u32;
        game.secrets = world
            .sites
            .iter()
            .filter(|site| site.kind == SiteKind::Secret)
            .count() as u32;
        game.checkpoint = world
            .sites
            .iter()
            .find(|site| site.kind == SiteKind::VeilAbility)
            .unwrap()
            .position;
        for gate in &mut world.gates {
            gate.latched = matches!(gate.kind, GateKind::Light | GateKind::Anchor);
            gate.open = if gate.latched { 1. } else { 0. };
        }
        (game, world)
    }

    #[test]
    fn roundtrip_restores_progress_and_preserves_completed_ending() {
        let (original, world) = finished(u64::MAX);
        let saved = Save::snapshot(&original, &world);
        let text = serde_json::to_string(&saved).unwrap();
        let decoded = decode(&text).unwrap();
        assert_eq!(saved, decoded);
        let (mut restored, mut regenerated) = fresh(u64::MAX);
        restore(Some(decoded), &mut restored, &mut regenerated);
        assert!(restored.started);
        assert_eq!(restored.mode, Mode::Title);
        assert_eq!(restored.player, original.checkpoint);
        assert_eq!(restored.camera, original.checkpoint);
        assert_eq!(restored.elapsed, original.elapsed);
        assert_eq!(restored.resonators, original.resonators);
        assert_eq!(restored.secrets, original.secrets);
        assert_eq!(restored.deaths, original.deaths);
        assert_eq!(restored.pulses, original.pulses);
        assert_eq!(restored.visited, original.visited);
        assert!(
            regenerated
                .sites
                .iter()
                .any(|site| site.kind == SiteKind::Ending && site.active)
        );
        assert_eq!(Save::snapshot(&restored, &regenerated), saved);
    }

    #[test]
    fn bad_schema_and_invalid_numbers_are_rejected() {
        let (game, world) = fresh(0);
        let valid = serde_json::to_value(Save::snapshot(&game, &world)).unwrap();
        for (field, value) in [
            ("version", serde_json::json!(99)),
            ("elapsed", serde_json::json!(-1)),
            ("elapsed", serde_json::json!(null)),
            ("checkpoint", serde_json::json!([0])),
            ("visited", serde_json::json!([1, 0])),
            ("seed", serde_json::json!(-1)),
            ("deaths", serde_json::json!(u32::MAX)),
            ("pulses", serde_json::json!(u32::MAX)),
        ] {
            let mut corrupted = valid.clone();
            corrupted[field] = value;
            assert!(
                decode(&corrupted.to_string()).is_none(),
                "accepted invalid {field}"
            );
        }
        assert!(decode("{truncated").is_none());
        assert!(decode("{}").is_none());
        assert!(decode(&" ".repeat(MAX_SAVE_BYTES + 1)).is_none());
        let mut nonfinite = Save::snapshot(&game, &world);
        nonfinite.elapsed = f32::INFINITY;
        assert!(!nonfinite.valid_for(&world));
        nonfinite.elapsed = 0.;
        nonfinite.checkpoint[0] = f32::NAN;
        assert!(!nonfinite.valid_for(&world));
    }

    #[test]
    fn contradictory_progress_and_dimensions_never_partially_restore() {
        let (original, source) = finished(72419);
        let good = Save::snapshot(&original, &source);
        let mut malformed = Vec::new();
        let mut bad = good.clone();
        bad.seed += 1;
        malformed.push(bad);
        let mut bad = good.clone();
        bad.visited.pop();
        malformed.push(bad);
        let mut bad = good.clone();
        bad.sites.pop();
        malformed.push(bad);
        let mut bad = good.clone();
        bad.gates.pop();
        malformed.push(bad);
        let mut bad = good.clone();
        bad.anchor = false;
        malformed.push(bad);
        let mut bad = good.clone();
        bad.veil = false;
        malformed.push(bad);
        let mut bad = good.clone();
        bad.resonators -= 1;
        malformed.push(bad);
        let mut bad = good.clone();
        bad.secrets -= 1;
        malformed.push(bad);
        let mut bad = good.clone();
        bad.checkpoint = source.rooms[23].center.to_array();
        malformed.push(bad);
        let mut bad = good.clone();
        bad.checkpoint = [1e30, -1e30];
        malformed.push(bad);
        let mut bad = good.clone();
        let transient = source
            .gates
            .iter()
            .position(|gate| gate.kind == GateKind::Shade)
            .unwrap();
        bad.gates[transient] = true;
        malformed.push(bad);
        for bad in malformed {
            let (mut game, mut world) = fresh(72419);
            let before = Save::snapshot(&game, &world);
            restore(Some(bad), &mut game, &mut world);
            assert!(!game.started);
            assert_eq!(game.player, world.spawn);
            assert_eq!(Save::snapshot(&game, &world), before);
        }
    }

    #[test]
    fn opening_save_keeps_needed_lessons_and_transient_gates_closed() {
        let (mut game, mut world) = fresh(123);
        game.started = true;
        game.pulses = 1;
        game.visited[0] = true;
        let light = world
            .gates
            .iter_mut()
            .find(|gate| gate.kind == GateKind::Light)
            .unwrap();
        light.latched = true;
        let saved = Save::snapshot(&game, &world);
        let (mut restored, mut regenerated) = fresh(123);
        restore(Some(saved), &mut restored, &mut regenerated);
        assert!(restored.started);
        assert_eq!(restored.hint_stage, 2);
        assert!(
            regenerated
                .gates
                .iter()
                .filter(|gate| gate.kind == GateKind::Shade)
                .all(|gate| !gate.latched && gate.open == 0.)
        );
    }

    #[test]
    fn ending_and_checkpoints_require_their_actual_world_progress() {
        let (game, world) = fresh(719);
        let clean = Save::snapshot(&game, &world);
        let mut premature_ending = clean.clone();
        let ending = world
            .sites
            .iter()
            .position(|site| site.kind == SiteKind::Ending)
            .unwrap();
        premature_ending.sites[ending] = true;
        assert!(!premature_ending.valid_for(&world));

        let mut untouched_checkpoint = clean.clone();
        let sanctuary = world
            .sites
            .iter()
            .find(|site| site.kind == SiteKind::Sanctuary)
            .unwrap();
        untouched_checkpoint.checkpoint = sanctuary.position.to_array();
        assert!(!untouched_checkpoint.valid_for(&world));

        let mut missing_anchor = clean;
        let anchor_gate = world
            .gates
            .iter()
            .position(|gate| gate.kind == GateKind::Anchor)
            .unwrap();
        missing_anchor.gates[anchor_gate] = true;
        assert!(!missing_anchor.valid_for(&world));
    }

    #[test]
    fn a_resonator_checkpoint_restores_only_after_activation() {
        let (game, world) = fresh(72419);
        let mut saved = Save::snapshot(&game, &world);
        let index = world
            .sites
            .iter()
            .position(|s| s.room == 14 && s.kind == SiteKind::Resonator)
            .unwrap();
        saved.checkpoint = world.sites[index].position.to_array();
        assert!(!saved.valid_for(&world));
        saved.sites[index] = true;
        saved.resonators = 1;
        assert!(saved.valid_for(&world));
    }

    #[test]
    fn browser_seed_parser_and_storage_slot_are_stable() {
        assert_eq!(STORAGE_KEY, "lastlight-save-v1");
        assert_eq!(query_seed("?seed=0"), Some(0));
        assert_eq!(
            query_seed("?sound=off&seed=18446744073709551615"),
            Some(u64::MAX)
        );
        assert_eq!(query_seed("?seed=invalid&seed=17"), Some(17));
        assert_eq!(query_seed("?seed=-1"), None);
        assert_eq!(query_seed("?seed=18446744073709551616"), None);
        assert_eq!(query_seed("?notseed=12"), None);
        assert_eq!(query_seed(""), None);
        let (game, world) = fresh(42);
        let saved = Save::snapshot(&game, &world);
        let before = serde_json::to_string(&saved).unwrap();
        let (mut other_game, mut other_world) = fresh(query_seed("?seed=99").unwrap());
        restore(Some(saved.clone()), &mut other_game, &mut other_world);
        assert!(!other_game.started);
        assert_eq!(serde_json::to_string(&saved).unwrap(), before);
    }

    #[test]
    fn legacy_geography_migrates_by_landmark_without_losing_progress() {
        let old = WorldMap::legacy(565069);
        let mut game = Game::new(old.seed, old.spawn, old.rooms.len());
        let mut old = old;
        game.has_anchor = true;
        game.resonators = 2;
        game.deaths = 11;
        for site in &mut old.sites {
            site.active = site.kind == SiteKind::AnchorAbility
                || site.room == 18
                || (site.kind == SiteKind::Resonator && site.room != 16);
        }
        game.checkpoint = old.rooms[18].center;
        let mut saved = Save::snapshot(&game, &old);
        saved.world_revision = 0;
        let mut new = WorldMap::generate(old.seed);
        let mut restored = Game::new(new.seed, new.spawn, new.rooms.len());
        restore(Some(saved), &mut restored, &mut new);
        assert!(restored.started && restored.has_anchor);
        assert_eq!(restored.resonators, 2);
        assert_eq!(restored.deaths, 11);
        assert_eq!(restored.checkpoint, new.rooms[18].center);
        assert_ne!(restored.checkpoint, game.checkpoint);
    }
    #[test]
    fn discoveries_roundtrip_and_old_saves_default_to_empty_pockets() {
        let (mut game, world) = fresh(72419);
        game.collected = 1 | 16;
        game.bell_found = true;
        game.oil = 1;
        game.charted = 7;
        let saved = Save::snapshot(&game, &world);
        let (mut restored, mut map) = fresh(72419);
        restore(Some(saved.clone()), &mut restored, &mut map);
        assert_eq!(
            (
                restored.collected,
                restored.bell_found,
                restored.oil,
                restored.charted
            ),
            (17, true, 1, 7)
        );
        let mut json = serde_json::to_value(saved).unwrap();
        for key in ["collected", "bell_found", "oil", "charted"] {
            json.as_object_mut().unwrap().remove(key);
        }
        let old = decode(&json.to_string()).unwrap();
        let (mut restored, mut map) = fresh(72419);
        restore(Some(old), &mut restored, &mut map);
        assert_eq!(
            (
                restored.collected,
                restored.bell_found,
                restored.oil,
                restored.charted
            ),
            (0, false, 0, 0)
        );
    }
}
