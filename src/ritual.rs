//! The folded structure's feedback uses exactly the same rules as its interaction.
use crate::{model::*, world::*};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldedState {
    NeedsAbility,
    NeedsLantern,
    TooClose,
    TooFar,
    Extinguish,
    PulseFading,
    Approach,
    Ready,
}

pub fn state(game: &Game, center: Vec2) -> FoldedState {
    if !game.has_anchor {
        return FoldedState::NeedsAbility;
    }
    let Some(anchor) = game.anchor else {
        return FoldedState::NeedsLantern;
    };
    if anchor.distance(center) < 95. {
        return FoldedState::TooClose;
    }
    if anchor.distance(center) >= 220. {
        return FoldedState::TooFar;
    }
    if game.brightness >= 0.1 {
        return FoldedState::Extinguish;
    }
    if game.pulse > 0. {
        return FoldedState::PulseFading;
    }
    if game.player.distance(center) > 85. {
        return FoldedState::Approach;
    }
    FoldedState::Ready
}

/// A suggested position, not a socket: every placement within the band works.
/// Keep enough clearance for the player's body and the 18-unit placement offset.
pub fn lantern_mark(world: &WorldMap, center: Vec2) -> Option<Vec2> {
    for step in 0..24 {
        let angle = -std::f32::consts::FRAC_PI_4 + step as f32 * std::f32::consts::TAU / 24.;
        let p = center + Vec2::from_angle(angle) * 155.;
        if world.field(p) > 45. && line_clear(world, center, p) {
            return Some(p);
        }
    }
    None
}

pub fn hint(game: &Game, world: &WorldMap) -> Option<&'static str> {
    let site = world.sites.iter().find(|s| {
        s.kind == SiteKind::Resonator
            && s.room == 16
            && !s.active
            && s.position.distance(game.player) < 300.
            && line_clear(world, game.player, s.position)
    })?;
    Some(match state(game, site.position) {
        FoldedState::NeedsAbility => {
            "This structure needs light from a placed lantern.\nFind the lantern placement ability, then return."
        }
        FoldedState::NeedsLantern => {
            "E    set a lantern on the small lantern mark beside the structure\nThen return to the centre, extinguish with X and hold R."
        }
        FoldedState::TooClose => {
            "Your placed lantern is too close to the centre.\nE    place it farther away, near the small lantern mark."
        }
        FoldedState::TooFar => {
            "Your placed lantern is too far from the structure.\nE    place it closer, near the small lantern mark."
        }
        FoldedState::Extinguish => {
            "The placed lantern is in reach. Leave it burning.\nX    extinguish your own light, then hold R at the centre."
        }
        FoldedState::PulseFading => {
            "Wait for your pulse to fade. Leave the placed lantern burning.\nThen hold R at the centre while you stay dark."
        }
        FoldedState::Approach => {
            "The lantern is in reach. Stay dark and approach the centre.\nHold R there to wake the structure."
        }
        FoldedState::Ready => {
            "Hold R    wake the structure\nStay dark and leave the placed lantern burning."
        }
    })
}

/// xy = suggested lantern location, z = guide visible, w = interaction ready.
pub fn visual(game: &Game, world: &WorldMap) -> Vec4 {
    let Some(site) = world
        .sites
        .iter()
        .find(|s| s.kind == SiteKind::Resonator && s.room == 16 && !s.active)
    else {
        return Vec4::ZERO;
    };
    if game.player.distance(site.position) > 360. {
        return Vec4::ZERO;
    }
    let Some(mark) = lantern_mark(world, site.position) else {
        return Vec4::ZERO;
    };
    Vec4::new(
        mark.x,
        mark.y,
        1.,
        f32::from(state(game, site.position) == FoldedState::Ready),
    )
}
