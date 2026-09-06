//! Branching roots share their retracting geometry with movement queries.
use crate::{model::*, world::*};
use bevy::prelude::*;
#[derive(Clone, Copy)]
pub struct Segment {
    pub a: Vec2,
    pub b: Vec2,
    pub ra: f32,
    pub rb: f32,
}
pub struct Patch {
    pub center: Vec2,
    pub origin: Vec2,
    pub extension: f32,
    hold: f32,
    segments: Vec<Segment>,
}
#[derive(Resource, Default)]
pub struct Rootwork {
    seed: u64,
    pub patches: Vec<Patch>,
}
fn random(n: f32) -> f32 {
    (n.sin() * 43758.547).fract().abs()
}
impl Rootwork {
    pub fn generate(world: &WorldMap) -> Self {
        let mut centers = Vec::new();
        for passage in &world.passages {
            for (room, index) in [(passage.a, 1), (passage.b, 4)] {
                if world.rooms[room].region == 1 {
                    let p = passage.points[index];
                    if centers.iter().all(|q: &Vec2| q.distance(p) > 180.) {
                        centers.push(p);
                    }
                }
            }
        }
        let patches = centers
            .into_iter()
            .enumerate()
            .map(|(id, center)| {
                let mut origin = center + Vec2::Y * 170.;
                let mut nearest = 200.;
                for i in 0..24 {
                    let dir = Vec2::from_angle(i as f32 * std::f32::consts::TAU / 24.);
                    for step in 3..18 {
                        let d = step as f32 * 12.;
                        let p = center + dir * d;
                        if world.field(p) < 0. {
                            if d < nearest {
                                origin = p;
                                nearest = d;
                            }
                            break;
                        }
                    }
                }
                let axis = (center - origin).normalize_or_zero();
                let tangent = Vec2::new(-axis.y, axis.x);
                let mut segments = Vec::new();
                for branch in 0..3 {
                    let seed = id as f32 * 13.7 + branch as f32 * 7.1 + world.seed as f32 % 997.;
                    let spread = (branch as f32 - 1.) * 66. + random(seed) * 27.;
                    let tip = center + axis * (35. + random(seed + 1.) * 55.) + tangent * spread;
                    let control = origin
                        + axis * nearest * 0.6
                        + tangent * (spread * 0.2 + (random(seed + 4.) - 0.5) * 75.);
                    let mut previous = origin;
                    for i in 1..=5 {
                        let t = i as f32 / 5.;
                        let b =
                            origin * (1. - t).powi(2) + control * (2. * t * (1. - t)) + tip * t * t;
                        let a = previous;
                        let ra =
                            (1. - (i - 1) as f32 / 5.).powi(2) * (7. + branch as f32 * 1.2) + 0.5;
                        let rb = (1. - t).powi(2) * (7. + branch as f32 * 1.2) + 0.4;
                        segments.push(Segment { a, b, ra, rb });
                        if i == 2 || i == 4 {
                            let spur = b
                                + axis * (15. + random(seed + i as f32) * 25.)
                                + tangent
                                    * (if i == 2 { -1. } else { 1. })
                                    * (22. + random(seed + 8.) * 35.);
                            let mid = b.lerp(spur, 0.55) + axis * 10.;
                            segments.push(Segment {
                                a: b,
                                b: mid,
                                ra: rb * 0.65,
                                rb: 1.2,
                            });
                            segments.push(Segment {
                                a: mid,
                                b: spur,
                                ra: 1.2,
                                rb: 0.2,
                            });
                        }
                        previous = b;
                    }
                }
                Patch {
                    center,
                    origin,
                    extension: 1.,
                    hold: 0.,
                    segments,
                }
            })
            .collect();
        Self {
            seed: world.seed,
            patches,
        }
    }
    pub fn dragging(&self, p: Vec2) -> bool {
        self.patches
            .iter()
            .filter(|r| r.extension > 0.45 && r.center.distance(p) < 250.)
            .any(|r| {
                r.visible_segments()
                    .iter()
                    .any(|s| segment_distance(p, s.a, s.b) < s.ra.max(s.rb) + 10.)
            })
    }
}
impl Patch {
    pub fn visible_segments(&self) -> Vec<Segment> {
        let transform = |p: Vec2| self.origin + (p - self.origin) * (0.06 + 0.94 * self.extension);
        self.segments
            .iter()
            .map(|s| Segment {
                a: transform(s.a),
                b: transform(s.b),
                ra: s.ra,
                rb: s.rb,
            })
            .collect()
    }
}
pub fn update(time: Res<Time>, world: Res<WorldMap>, game: Res<Game>, mut roots: ResMut<Rootwork>) {
    if roots.seed != world.seed || roots.patches.is_empty() {
        *roots = Rootwork::generate(&world);
    }
    if game.mode != Mode::Playing || game.map_open {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    for patch in &mut roots.patches {
        let direct = (game.brightness > 0.25 && patch.center.distance(game.player) < 145.)
            || (game.pulse > 0.2 && patch.center.distance(game.player) < 390.);
        let lit = direct && line_clear(&world, game.player, patch.center)
            || game.anchor.is_some_and(|a| {
                a.distance(patch.center) < 230. && line_clear(&world, a, patch.center)
            });
        if lit {
            patch.hold = 2.8;
        } else {
            patch.hold = (patch.hold - dt).max(0.);
        }
        let target = if patch.hold > 0. { 0. } else { 1. };
        patch.extension +=
            (target - patch.extension) * (1. - (-dt * if target == 0. { 6. } else { 0.8 }).exp());
    }
}
