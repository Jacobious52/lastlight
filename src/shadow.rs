//! The large organism is a spreading volume, not a scaled walking sprite.
//! Collision and rendering consume the same tapered branch segments.
use crate::{model::Creature, world::segment_distance};
use bevy::prelude::*;

pub const SEGMENTS: usize = 56;

pub fn width(index: usize) -> f32 {
    0.3 + 68. * (1. - index as f32 / 8.).max(0.).powi(2)
}

pub fn branches(c: &Creature) -> [Vec4; SEGMENTS] {
    let root = c.home + Vec2::Y * 380.;
    std::array::from_fn(|i| {
        let branch = i / 8;
        let spread = (branch as f32 - 3.) * 74.;
        let tip = c.position + Vec2::new(spread, -28. + (branch as f32 * 1.9).sin() * 75.);
        let root = root + Vec2::X * spread * 0.45;
        let control = root + Vec2::new(spread * 0.7, -135.);
        let hook = tip
            + Vec2::new(
                (branch as f32 * 2.3).cos() * 150. + (c.phase * 0.4 + branch as f32).sin() * 38.,
                90.,
            );
        let curve = |t: f32| {
            root * (1. - t).powi(3)
                + control * (3. * t * (1. - t).powi(2))
                + hook * (3. * t * t * (1. - t))
                + tip * t.powi(3)
        };
        let a = curve((i % 8) as f32 / 8.);
        let b = curve((i % 8 + 1) as f32 / 8.);
        Vec4::new(a.x, a.y, b.x, b.y)
    })
}

pub fn distance(c: &Creature, p: Vec2) -> f32 {
    branches(c)
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let ab = s.zw() - s.xy();
            let t = ((p - s.xy()).dot(ab) / ab.length_squared().max(0.01)).clamp(0., 1.);
            let radius = width(i % 8) + (width(i % 8 + 1) - width(i % 8)) * t;
            segment_distance(p, s.xy(), s.zw()) - radius
        })
        .fold(f32::INFINITY, f32::min)
}
