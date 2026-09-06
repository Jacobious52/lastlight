//! A spatial grammar, rather than a bag of randomly connected rooms.
//!
//! Room roles stay stable so the opening and ability lessons can be authored.
//! Landmarks sit inside fractured galleries, gullies and crooked passages.
//! The progression graph is stable; its physical embedding uses concave rock
//! shelves and variable-width polylines, with protected throats around gates.
use bevy::prelude::*;

const EDGE_ROUGHNESS: f32 = 9.0;

#[derive(Resource, Clone)]
pub struct WorldMap {
    pub seed: u64,
    pub rooms: Vec<Room>,
    pub passages: Vec<Passage>,
    pub gates: Vec<Gate>,
    pub sites: Vec<Site>,
    pub spawn: Vec2,
}

#[derive(Clone, Debug)]
pub struct Room {
    pub id: usize,
    pub center: Vec2,
    pub radius: Vec2,
    pub region: u32,
    pub name: &'static str,
    pub formations: Vec<Formation>,
    pub boundary: Vec<Vec2>,
}

/// A solid outcrop or fallen length of material. Its silhouette and collision
/// come from the same distance field; it can shelter a dark player from sight.
#[derive(Clone, Debug)]
pub struct Formation {
    pub a: Vec2,
    pub b: Vec2,
    pub radius: f32,
}

/// `width` is the *half*-width of this capsule, in world units.
#[derive(Clone, Debug)]
pub struct Passage {
    pub a: usize,
    pub b: usize,
    pub width: f32,
    pub points: Vec<Vec2>,
    pub widths: Vec<f32>,
    pub gate: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Gate {
    pub position: Vec2,
    /// Direction of travel through the gate, not direction along its surface.
    pub normal: Vec2,
    pub half_width: f32,
    pub kind: GateKind,
    pub open: f32,
    pub latched: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateKind {
    Light,
    Shade,
    Anchor,
    Veil,
    Final,
}

#[derive(Clone, Debug)]
pub struct Site {
    pub position: Vec2,
    pub room: usize,
    pub kind: SiteKind,
    pub active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiteKind {
    Sanctuary,
    AnchorAbility,
    VeilAbility,
    Resonator,
    Secret,
    Ending,
}

/// Small local PRNG: save seeds are independent of crate or platform versions.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn range(&mut self, low: f32, high: f32) -> f32 {
        low + ((self.next() >> 40) as f32 / 16_777_216.0) * (high - low)
    }
    fn coin(&mut self) -> bool {
        self.next() & 1 != 0
    }
}

impl WorldMap {
    pub fn generate(seed: u64) -> Self {
        Self::generate_layout(seed, false)
    }

    pub fn legacy(seed: u64) -> Self {
        Self::generate_layout(seed, true)
    }

    fn generate_layout(seed: u64, legacy: bool) -> Self {
        // Bounded rejection is a guard against future grammar changes. Geometry
        // is never accepted merely because its abstract graph is connected.
        for attempt in 0..12_u64 {
            let candidate = Self::candidate(seed, attempt, legacy);
            if candidate.validate().is_ok() {
                return candidate;
            }
        }
        // The zero-jitter fallback has the same seed/progression and safe spacing.
        // Its generous separation is also verified in the generator tests.
        let fallback = Self::candidate(seed, 12, legacy);
        debug_assert!(fallback.validate().is_ok(), "invalid fallback world");
        fallback
    }

    fn candidate(seed: u64, attempt: u64, legacy: bool) -> Self {
        let mut rng = Rng(seed.wrapping_add(attempt.wrapping_mul(0xd134_2543_de82_ef95)));
        // Legacy landmark coordinates retain a migration reference. New worlds
        // use the oblique embedding below while preserving progression roles.
        let mut templates = [
            (-3000., 0., 305., 250., 0, "Cradle"),
            (-2180., 0., 280., 290., 0, "Fold"),
            (-1360., 0., 310., 330., 0, "Witness"),
            (-500., 0., 385., 365., 0, "Hollow"),
            (-500., 880., 300., 330., 1, "Listening Well"),
            (390., 880., 340., 300., 1, "Split Nerve"),
            (1260., 880., 315., 315., 1, "Still Hearth"),
            (1260., 0., 285., 325., 2, "Rest"),
            (390., 0., 300., 300., 0, "Return"),
            (-500., -880., 310., 325., 3, "Curtain"),
            (390., -880., 325., 295., 3, "Absent Garden"),
            (1260., -880., 305., 325., 3, "Afterimage"),
            (2130., -880., 325., 350., 2, "Suspension"),
            (2130., 0., 320., 340., 2, "Long Gallery"),
            (2130., 880., 350., 325., 2, "Choir"),
            (1260., 1760., 325., 285., 1, "High Crossing"),
            (390., 1760., 310., 315., 1, "Sealed Lung"),
            (-500., -1760., 305., 300., 3, "Underside"),
            (390., -1760., 325., 310., 3, "Quiet Pool"),
            (1260., -1760., 340., 330., 3, "Unlit Span"),
            (-1360., -880., 265., 310., 3, "Old Passage"),
            (2130., 1760., 305., 290., 2, "Blind Crown"),
            (2130., -1760., 350., 320., 3, "The Weight"),
            (-1360., 880., 300., 320., 0, "Last Chamber"),
        ];
        // Offset, oblique circuits. The old embedding is retained only for
        // validating and migrating existing saves by landmark identity.
        if !legacy {
            let centers = [
                (-3000., 0.),
                (-2180., 0.),
                (-1360., 0.),
                (-450., -70.),
                (-560., 1000.),
                (220., 1400.),
                (1110., 1420.),
                (1140., 480.),
                (280., 210.),
                (-450., -1130.),
                (590., -1140.),
                (1710., -690.),
                (2680., -240.),
                (2120., 690.),
                (2080., 1720.),
                (1130., 2370.),
                (80., 2260.),
                (-610., -2140.),
                (420., -2460.),
                (1490., -2090.),
                (-1620., -1040.),
                (2130., 2760.),
                (2590., -1570.),
                (-1670., 850.),
            ];
            for (template, (x, y)) in templates.iter_mut().zip(centers) {
                template.0 = x;
                template.1 = y;
            }
        }
        let stable = attempt == 12;
        let rooms: Vec<_> = templates
            .into_iter()
            .enumerate()
            .map(|(id, (x, y, rx, ry, region, name))| {
                let jitter = if stable || id < 3 {
                    0.
                } else if legacy {
                    28.
                } else {
                    65.
                };
                let scale = if stable { 1. } else { rng.range(0.93, 1.035) };
                Room {
                    id,
                    center: Vec2::new(
                        x + rng.range(-jitter, jitter),
                        y + rng.range(-jitter, jitter),
                    ),
                    radius: Vec2::new(rx, ry) * scale,
                    region,
                    name,
                    formations: vec![],
                    boundary: {
                        // Long fractured shelves with recesses and angular ends.
                        // The hub and two deep landmarks retain generous volume.
                        let broad = matches!(id, 3 | 14 | 19 | 23);
                        let mut shape_rng = Rng(seed ^ (id as u64 + 1).wrapping_mul(7919));
                        let shelf = [
                            (-0.97, -0.24),
                            (-0.60, -0.37),
                            (-0.27, -0.27),
                            (0.03, -0.67),
                            (0.47, -0.73),
                            (0.91, -0.28),
                            (0.80, 0.30),
                            (0.45, 0.26),
                            (0.18, 0.81),
                            (-0.17, 0.71),
                            (-0.41, 0.34),
                            (-0.91, 0.40),
                        ];
                        let cleft = [
                            (-0.97, -0.23),
                            (-0.63, -0.30),
                            (-0.23, -0.19),
                            (0.15, -0.32),
                            (0.52, -0.43),
                            (0.96, -0.26),
                            (0.88, 0.20),
                            (0.46, 0.25),
                            (0.08, 0.13),
                            (-0.32, 0.38),
                            (-0.65, 0.31),
                            (-0.92, 0.22),
                        ];
                        let gallery = [
                            (-0.92, -0.50),
                            (-0.22, -0.56),
                            (-0.22, -0.26),
                            (0.46, -0.26),
                            (0.46, -0.58),
                            (0.91, -0.49),
                            (0.94, 0.56),
                            (0.29, 0.56),
                            (0.29, 0.27),
                            (-0.49, 0.27),
                            (-0.49, 0.64),
                            (-0.92, 0.57),
                        ];
                        let rift = [
                            (-0.28, -0.93),
                            (0.10, -0.89),
                            (0.27, -0.48),
                            (0.19, -0.12),
                            (0.68, 0.20),
                            (0.49, 0.66),
                            (0.08, 0.93),
                            (-0.23, 0.64),
                            (-0.24, 0.34),
                            (-0.18, 0.04),
                            (-0.62, -0.19),
                            (-0.45, -0.57),
                        ];
                        let profile = match id {
                            1 | 8 | 20 => cleft,
                            12 | 13 | 15 | 21 => gallery,
                            4 | 9 | 11 | 17 => rift,
                            _ => shelf,
                        };
                        profile
                            .into_iter()
                            .map(|(px, py)| {
                                let stretch = if broad { 1. } else { 0.73 };
                                Vec2::new(x, y)
                                    + Vec2::new(px * rx, py * ry * stretch)
                                        * shape_rng.range(0.90, 1.04)
                            })
                            .collect()
                    },
                }
            })
            .collect();
        let mut rooms = rooms;
        for room in &mut rooms {
            let (x, y, ..) = templates[room.id];
            let shift = room.center - Vec2::new(x, y);
            for point in &mut room.boundary {
                *point += shift;
            }
        }
        let spawn = rooms[0].center + Vec2::new(-110., 0.);
        let mut world = Self {
            seed,
            rooms,
            passages: vec![],
            gates: vec![],
            sites: vec![],
            spawn,
        };
        use GateKind::*;
        let grammar = [
            // Guarded opening. No accessible side exit can skip these lessons.
            (0, 1, Some(Light)),
            (1, 2, Some(Shade)),
            (2, 3, None),
            // First journey circles back through a formerly impassable seam.
            (3, 4, None),
            (4, 5, None),
            (5, 6, None),
            (6, 7, None),
            (7, 8, Some(Anchor)),
            (8, 3, None),
            // Anchor is useful before the player discovers the Veil lesson.
            (3, 9, Some(Anchor)),
            (9, 10, None),
            (10, 11, Some(Veil)),
            (11, 12, Some(Shade)),
            (12, 13, None),
            (13, 14, None),
            (14, 6, Some(Light)),
            // Old northern landmarks become penetrable after learning Veil.
            (5, 16, Some(Veil)),
            (16, 15, Some(Veil)),
            (15, 21, None),
            (21, 14, Some(Anchor)),
            // A second deep loop offers two approaches to the large encounter.
            (9, 17, Some(Veil)),
            (17, 18, None),
            (18, 19, Some(Shade)),
            (19, 22, Some(Anchor)),
            (22, 12, None),
            // Optional shortcut reconnects the depths with the early opening.
            (9, 20, Some(Veil)),
            (20, 2, Some(Veil)),
            // The conclusion is physically beside the familiar central haven.
            (3, 23, Some(Final)),
        ];
        for (a, b, gate) in grammar {
            world.connect(a, b, gate, rng.range(59., 73.));
        }
        // These chords make routes differ between seeds without changing which
        // abilities the world promises or letting one bypass the first lessons.
        if rng.coin() {
            world.connect(5, 8, Some(Light), rng.range(58., 72.));
        }
        if rng.coin() {
            world.connect(6, 15, Some(Light), rng.range(59., 72.));
        }
        if rng.coin() {
            world.connect(7, 13, Some(Shade), rng.range(59., 74.));
        }
        if rng.coin() {
            world.connect(10, 18, Some(Anchor), rng.range(60., 74.));
        }
        use SiteKind::*;
        for (room, kind) in [
            (0, Sanctuary),
            (3, Sanctuary),
            (6, AnchorAbility),
            (7, Sanctuary),
            (10, VeilAbility),
            (14, Resonator),
            (16, Resonator),
            (18, Sanctuary),
            (20, Secret),
            (21, Secret),
            (22, Resonator),
            (23, Ending),
        ] {
            world.sites.push(Site {
                position: world.rooms[room].center,
                room,
                kind,
                active: false,
            });
        }
        // Outcrops occupy actual floor outside the winding navigable spine.
        // Their painted props, collision and light obstruction share placement.
        for id in 0..world.rooms.len() {
            let center = world.rooms[id].center;
            let radius = world.rooms[id].radius;
            let routes: Vec<(Vec2, Vec2)> = world
                .passages
                .iter()
                .filter(|p| p.a == id || p.b == id)
                .flat_map(|p| p.points.windows(2).map(|w| (w[0], w[1])))
                .collect();
            for _ in 0..96 {
                let a = center + Vec2::new(rng.range(-0.88, 0.88), rng.range(-0.74, 0.74)) * radius;
                let axis = Vec2::from_angle(rng.range(0., std::f32::consts::TAU));
                let b = a + axis * rng.range(20., 65.);
                let thickness = rng.range(10., 23.);
                if world.field((a + b) * 0.5) < thickness + 7.
                    || segment_distance(center, a, b) < 82. + thickness
                    || segment_distance(world.spawn, a, b) < 48. + thickness
                    || routes.iter().any(|(start, end)| {
                        segment_separation(*start, *end, a, b) < 62. + thickness
                    })
                    || (id == 16
                        && segment_separation(
                            center - Vec2::X * 170.,
                            center + Vec2::Y * 170.,
                            a,
                            b,
                        ) < 42. + thickness)
                {
                    continue;
                }
                world.rooms[id].formations.push(Formation {
                    a,
                    b,
                    radius: thickness,
                });
            }
        }
        world
    }

    fn connect(&mut self, a: usize, b: usize, kind: Option<GateKind>, width: f32) {
        let start = self.rooms[a].center;
        let end = self.rooms[b].center;
        let normal = (end - start).normalize();
        let tangent = Vec2::new(-normal.y, normal.x);
        let mut rng = Rng(self.seed ^ ((a * 129 + b * 179) as u64));
        // A short straight throat seals the gate; the approach bends out of sight.
        // Keep the diagonal ending passage inside its reserved planar envelope.
        let bend = if a == 3 && b == 23 {
            0.
        } else {
            rng.range(72., 132.) * if rng.coin() { 1. } else { -1. }
        };
        let points = vec![
            start,
            start.lerp(end, 0.23) + tangent * bend,
            start.lerp(end, 0.38),
            start.lerp(end, 0.62),
            start.lerp(end, 0.77) - tangent * bend * 0.72,
            end,
        ];
        let widths = vec![
            105.,
            rng.range(85., 125.),
            width,
            width,
            rng.range(82., 120.),
            105.,
        ];
        let gate = kind.map(|kind| {
            let index = self.gates.len();
            self.gates.push(Gate {
                position: start.lerp(end, 0.5),
                normal,
                half_width: width + EDGE_ROUGHNESS + 16.,
                kind,
                open: 0.,
                latched: false,
            });
            index
        });
        self.passages.push(Passage {
            a,
            b,
            width,
            points,
            widths,
            gate,
        });
    }

    /// Signed navigable distance. Concave polygon shelves merge into varying
    /// width, bent passage spines. No room is represented by an ellipse.
    pub fn field(&self, p: Vec2) -> f32 {
        let mut field = -1000.0_f32;
        for room in &self.rooms {
            let outside = ((p - room.center).abs() - room.radius - Vec2::splat(22.))
                .max(Vec2::ZERO)
                .length();
            if outside > 0. && -outside < field {
                continue;
            }
            field = field.max(polygon_field(p, &room.boundary));
            // The indirect-light lesson needs a reachable place to leave a light.
            if room.id == 16 {
                let q = p - room.center - Vec2::Y * 80.;
                field = field.max((70. - q.x.abs()).min(145. - q.y.abs()));
            }
        }
        for passage in &self.passages {
            let a = self.rooms[passage.a].center;
            let b = self.rooms[passage.b].center;
            if segment_distance(p, a, b) > 280. && field > -120. {
                continue;
            }
            for (i, w) in passage.points.windows(2).enumerate() {
                let ab = w[1] - w[0];
                let t = ((p - w[0]).dot(ab) / ab.length_squared()).clamp(0., 1.);
                let width = passage.widths[i] + (passage.widths[i + 1] - passage.widths[i]) * t;
                field = field.max(width - p.distance(w[0] + ab * t));
            }
        }
        // Avoid expensive trigonometry in large uninterrupted interiors/rock.
        // Returning an unperturbed far field is continuous at these bounds,
        // since the roughness fades to zero at 90 units from the nominal edge.
        let edge_factor = (1. - field.abs() / 90.).max(0.);
        if edge_factor > 0. {
            let phase = (self.seed % 65521) as f32 * 0.0019;
            let rough = (p.x * 0.024 + (p.y * 0.011 + phase).sin() * 1.8).sin() * 4.5
                + (p.y * 0.038 - p.x * 0.017 + phase).sin() * 2.7
                + (p.x * 0.081 + p.y * 0.055).sin() * 1.5;
            field += rough * edge_factor;
        }
        // Interior outcrops break open arenas into passages, cover and alcoves.
        // Culling keeps collision and light rays cheap outside each chamber.
        for room in &self.rooms {
            if (p - room.center)
                .abs()
                .cmpgt(room.radius + Vec2::splat(120.))
                .any()
            {
                continue;
            }
            for rock in &room.formations {
                // Rounded collision follows the broad painted footprint; tiny
                // decorative chips no longer form hooks around the player's feet.
                let cut = segment_distance(p, rock.a, rock.b) - rock.radius;
                field = field.min(cut);
            }
        }
        field
    }

    pub fn creature_spawn(&self, room: usize) -> Vec2 {
        let center = self.rooms[room].center;
        [
            Vec2::new(150., 100.),
            Vec2::new(150., 0.),
            Vec2::new(-150., 0.),
            Vec2::new(0., 150.),
            Vec2::new(0., -150.),
            Vec2::ZERO,
        ]
        .into_iter()
        .map(|offset| center + offset)
        .find(|point| self.field(*point) > 38.)
        .unwrap_or(center)
    }

    pub fn room_at(&self, p: Vec2) -> usize {
        let nearest = self
            .rooms
            .iter()
            .min_by(|a, b| {
                a.center
                    .distance_squared(p)
                    .total_cmp(&b.center.distance_squared(p))
            })
            .map_or(0, |room| room.id);
        if polygon_field(p, &self.rooms[nearest].boundary) >= -12. {
            return nearest;
        }
        // A bent passage belongs to its endpoints. Euclidean landmark distance
        // alone mislabeled the final approach as Witness through solid rock.
        let mut closest = 140.;
        let mut owner = nearest;
        for passage in &self.passages {
            for segment in passage.points.windows(2) {
                let d = segment_distance(p, segment[0], segment[1]);
                if d < closest {
                    closest = d;
                    owner = if p.distance_squared(self.rooms[passage.a].center)
                        < p.distance_squared(self.rooms[passage.b].center)
                    {
                        passage.a
                    } else {
                        passage.b
                    };
                }
            }
        }
        owner
    }

    fn gate_for(&self, passage: &Passage) -> Option<&Gate> {
        passage.gate.and_then(|index| self.gates.get(index))
    }

    fn reachable(&self, anchor: bool, veil: bool, resonators: usize) -> Vec<bool> {
        let mut reached = vec![false; self.rooms.len()];
        reached[0] = true;
        loop {
            let mut changed = false;
            for passage in &self.passages {
                let passable = match self.gate_for(passage).map(|g| g.kind) {
                    Some(GateKind::Anchor) => anchor,
                    Some(GateKind::Veil) => veil,
                    Some(GateKind::Final) => resonators == 3,
                    _ => true,
                };
                if passable && reached[passage.a] != reached[passage.b] {
                    reached[passage.a] = true;
                    reached[passage.b] = true;
                    changed = true;
                }
            }
            if !changed {
                return reached;
            }
        }
    }

    /// Validate both the progression graph and the *physical* map. Graph-only
    /// validation misses intersecting tunnels and gates that an ellipse bypasses.
    pub fn validate(&self) -> Result<(), String> {
        if self.rooms.len() != 24 || self.field(self.spawn) < 35. {
            return Err("Missing rooms or obstructed spawn".into());
        }
        for (id, room) in self.rooms.iter().enumerate() {
            if room.id != id || room.radius.min_element() < 200. {
                return Err(format!("Invalid room {id}"));
            }
            for other in self.rooms.iter().skip(id + 1) {
                if room.center.distance(other.center)
                    < room.radius.max_element() + other.radius.max_element() + 24.
                {
                    return Err(format!("Overlapping rooms {id}/{}", other.id));
                }
            }
        }
        for (index, passage) in self.passages.iter().enumerate() {
            if passage.a >= self.rooms.len() || passage.b >= self.rooms.len() {
                return Err("Passage refers to missing room".into());
            }
            for (leg, points) in passage.points.windows(2).enumerate() {
                let width = passage.widths[leg].max(passage.widths[leg + 1]);
                for room in &self.rooms {
                    if room.id != passage.a
                        && room.id != passage.b
                        && segment_distance(room.center, points[0], points[1])
                            < room.radius.max_element() + width + EDGE_ROUGHNESS + 10.
                    {
                        return Err(format!("Passage {index} cuts unrelated room {}", room.id));
                    }
                }
                for other in self.passages.iter().skip(index + 1) {
                    if [passage.a, passage.b].contains(&other.a)
                        || [passage.a, passage.b].contains(&other.b)
                    {
                        continue;
                    }
                    for (j, w) in other.points.windows(2).enumerate() {
                        if segment_separation(points[0], points[1], w[0], w[1])
                            < width
                                + other.widths[j].max(other.widths[j + 1])
                                + EDGE_ROUGHNESS * 2.
                                + 12.
                        {
                            return Err(format!("Passage {index} makes an unintended junction"));
                        }
                    }
                }
                // Continuous clearance sampled at less than the player diameter.
                let samples = (points[0].distance(points[1]) / 20.).ceil() as usize;
                for sample in 0..=samples {
                    if self.field(points[0].lerp(points[1], sample as f32 / samples as f32)) < 36. {
                        return Err(format!("Obstructed passage {index}"));
                    }
                }
            }
            if let Some(gate) = self.gate_for(passage) {
                let tangent = Vec2::new(-gate.normal.y, gate.normal.x);
                if self.field(gate.position) < 35.
                    || gate.half_width < passage.width + EDGE_ROUGHNESS
                    || self.field(gate.position + tangent * (gate.half_width + 1.)) > 0.
                    || self.field(gate.position - tangent * (gate.half_width + 1.)) > 0.
                {
                    return Err(format!("Gate on passage {index} does not seal its tunnel"));
                }
            }
        }
        for site in &self.sites {
            if site.room >= self.rooms.len() || self.field(site.position) < 35. {
                return Err("Site outside navigable space".into());
            }
        }
        if self
            .sites
            .iter()
            .filter(|s| s.kind == SiteKind::Resonator)
            .count()
            != 3
        {
            return Err("World requires exactly three resonators".into());
        }
        let base = self.reachable(false, false, 0);
        let anchored = self.reachable(true, false, 0);
        let veiled = self.reachable(true, true, 0);
        let complete = self.reachable(true, true, 3);
        for site in &self.sites {
            let r = site.room;
            match site.kind {
                SiteKind::AnchorAbility if !base[r] => return Err("Anchor deadlock".into()),
                SiteKind::VeilAbility if base[r] || !anchored[r] => {
                    return Err("Veil must require Anchor and remain reachable".into());
                }
                SiteKind::Resonator if !veiled[r] => return Err("Unreachable resonator".into()),
                SiteKind::Ending if veiled[r] || !complete[r] => {
                    return Err("Conclusion must require all three resonators".into());
                }
                _ => {}
            }
        }
        if !self
            .sites
            .iter()
            .any(|s| s.kind == SiteKind::Resonator && !anchored[s.room])
        {
            return Err("Veil has no required use after acquisition".into());
        }
        if complete.iter().any(|&reachable| !reachable) {
            return Err("An area has no eventual route".into());
        }
        if self.passages.len() < self.rooms.len() {
            return Err("World has no loops".into());
        }
        Ok(())
    }
}

/// Exact edge distance and even/odd inside test for a simple concave polygon.
fn polygon_field(p: Vec2, vertices: &[Vec2]) -> f32 {
    let mut nearest = f32::MAX;
    let mut inside = false;
    for i in 0..vertices.len() {
        let a = vertices[i];
        let b = vertices[(i + 1) % vertices.len()];
        nearest = nearest.min(segment_distance(p, a, b));
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
    }
    if inside { nearest } else { -nearest }
}

pub fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let denominator = ab.length_squared();
    if denominator < 0.0001 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / denominator).clamp(0., 1.);
    p.distance(a + ab * t)
}

fn segment_separation(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> f32 {
    let ab = b - a;
    let cd = d - c;
    let cross = ab.perp_dot(cd);
    if cross.abs() > 0.001 {
        let t = (c - a).perp_dot(cd) / cross;
        let u = (c - a).perp_dot(ab) / cross;
        if (0. ..=1.).contains(&t) && (0. ..=1.).contains(&u) {
            return 0.;
        }
    }
    segment_distance(a, c, d)
        .min(segment_distance(b, c, d))
        .min(segment_distance(c, a, b))
        .min(segment_distance(d, a, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_of_seeds_have_physical_and_progression_integrity() {
        for seed in 0..2048 {
            let world = WorldMap::generate(seed);
            assert_eq!(world.validate(), Ok(()), "seed {seed}");
        }
        for seed in [u64::MAX, 0xdead_beef_f00d_cafe, 20260906] {
            assert_eq!(WorldMap::generate(seed).validate(), Ok(()));
        }
    }

    #[test]
    fn every_corridor_can_physically_be_traversed() {
        for seed in [0, 1, 17, 444, 20260906] {
            let world = WorldMap::generate(seed);
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
    fn fallback_geometry_is_safe() {
        for seed in 0..32 {
            assert_eq!(WorldMap::candidate(seed, 12, false).validate(), Ok(()));
        }
    }

    #[test]
    fn passages_retain_their_region_instead_of_borrowing_an_unconnected_landmark() {
        let world = WorldMap::generate(72419);
        for passage in &world.passages {
            for segment in passage.points.windows(2) {
                for i in 0..=8 {
                    let p = segment[0].lerp(segment[1], i as f32 / 8.);
                    assert!(
                        [passage.a, passage.b].contains(&world.room_at(p)),
                        "passage {}–{} mislabeled at {p:?}",
                        passage.a,
                        passage.b
                    );
                }
            }
        }
    }

    #[test]
    fn same_seed_reproduces_geometry_and_different_seeds_change_routes() {
        let a = WorldMap::generate(913);
        let b = WorldMap::generate(913);
        assert_eq!(a.passages.len(), b.passages.len());
        for (ra, rb) in a.rooms.iter().zip(&b.rooms) {
            assert_eq!(ra.center, rb.center);
            assert_eq!(ra.radius, rb.radius);
        }
        let signatures: std::collections::HashSet<Vec<_>> = (0..32)
            .map(|seed| {
                WorldMap::generate(seed)
                    .passages
                    .iter()
                    .map(|p| (p.a, p.b))
                    .collect()
            })
            .collect();
        assert!(
            signatures.len() >= 8,
            "Seed variation must change actual routes"
        );
    }

    #[test]
    fn gates_are_physically_impassable_around_their_ends() {
        for seed in 0..64 {
            let world = WorldMap::generate(seed);
            for gate in &world.gates {
                let tangent = Vec2::new(-gate.normal.y, gate.normal.x);
                for sign in [-1., 1.] {
                    for step in 0..12 {
                        let point =
                            gate.position + tangent * sign * (gate.half_width + step as f32 * 4.);
                        assert!(
                            world.field(point) < 0.,
                            "Gate can be skirted in seed {seed}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn segment_distance_handles_degenerate_segments() {
        assert_eq!(
            segment_distance(Vec2::new(3., 4.), Vec2::ZERO, Vec2::ZERO),
            5.
        );
        assert_eq!(
            segment_distance(Vec2::new(5., 4.), Vec2::ZERO, Vec2::new(10., 0.)),
            4.
        );
        assert_eq!(
            segment_separation(Vec2::ZERO, Vec2::ONE, Vec2::X, Vec2::Y),
            0.
        );
    }
}
