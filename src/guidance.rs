//! Context follows the obstruction, independently of one-off tutorial messages.
use crate::{model::*, world::*};

pub fn playing_hint(game: &Game, world: &WorldMap) -> String {
    if game.anchor_placing > 0. {
        return "Placing lantern; stay still".into();
    }
    if let Some(hint) = crate::ritual::hint(game, world) {
        return hint.into();
    }
    if let Some(hint) = gate_hint(game, world) {
        return hint;
    }
    if game.message_time > 0. {
        return game.message.clone();
    }
    opening_hint(game, world).unwrap_or_default().into()
}

fn gate_hint(game: &Game, world: &WorldMap) -> Option<String> {
    let gate = world
        .gates
        .iter()
        .filter(|gate| {
            if gate.latched || gate.position.distance(game.player) > 180. {
                return false;
            }
            let relative = game.player - gate.position;
            let side = if relative.dot(gate.normal) < 0. {
                -1.
            } else {
                1.
            };
            // Check the near face: a closed membrane blocks a ray to its centre.
            relative.perp_dot(gate.normal).abs() < gate.half_width + 30.
                && line_clear(world, game.player, gate.position + gate.normal * side * 32.)
        })
        .min_by(|a, b| {
            a.position
                .distance_squared(game.player)
                .total_cmp(&b.position.distance_squared(game.player))
        })?;
    let anchored = game
        .anchor
        .is_some_and(|a| a.distance(gate.position) < 180.);
    Some(match gate.kind {
        GateKind::Light => {
            "Space    pulse near the circular membrane to open it".into()
        }
        GateKind::Shade if anchored => {
            "This membrane needs darkness.\nR beside your placed lantern to extinguish it, then X to dim your own light.".into()
        }
        GateKind::Shade if game.brightness < 0.09 && game.pulse < 0.15 => {
            "Stay dark and pass through. X restores your light.".into()
        }
        GateKind::Shade => {
            "X / hold Shift    extinguish to open the thorned membrane\nWait for any pulse to fade, then cross in darkness.".into()
        }
        GateKind::Anchor if anchored => {
            "Leave the lantern burning and cross the opening".into()
        }
        GateKind::Anchor if game.has_anchor => {
            "E    place a lantern beside the ribbed membrane; stay still\nIt needs a placed lantern's steady light. A Space pulse is too brief.".into()
        }
        GateKind::Anchor => {
            "This ribbed membrane needs a placed lantern. A Space pulse is too brief.\nFind the lantern placement ability, then return here.".into()
        }
        GateKind::Veil if game.has_veil => {
            "Veil    X / hold Shift to extinguish, then walk through the woven curtain".into()
        }
        GateKind::Veil => {
            "Veil ability needed to cross this woven curtain.\nExplore another route and return when you have it.".into()
        }
        GateKind::Final if game.resonators >= 3 => "The passage is open. Step through.".into(),
        GateKind::Final => format!(
            "Wake {} more structures to open this passage.\nTab    view your route and progress",
            3 - game.resonators
        ),
    })
}

fn opening_hint(game: &Game, world: &WorldMap) -> Option<&'static str> {
    if game.hint_stage >= 4 {
        return None;
    }
    match game.room {
        0 if game.moved < 40. => Some("WASD / arrows    move       or click a destination"),
        0 if world.gates[0].latched => Some("The membrane stays open. Follow the passage."),
        0 if game.pulses == 0 => Some("Space    release a pulse to reveal the path ahead"),
        0 => Some("Approach the circular membrane. Release another pulse beside it."),
        1 if !game.bell_found => {
            Some("Look for a small bell in the light. Move close and press R to take it.")
        }
        1 if game.bell_out.is_some() => {
            Some("The bell is on the ground. Move close and press R to retrieve it.")
        }
        1 => Some(
            "Q    throw the bell ahead to draw attention away from you\nR    retrieve it when nearby",
        ),
        2 if game.dark_time > 1. && game.pulse < 0.15 => {
            Some("Stay dark while it searches. Move past when its path is clear.")
        }
        2 => Some(
            "Light draws the creature toward you.\nX / hold Shift    extinguish and move quietly past",
        ),
        _ => None,
    }
}
