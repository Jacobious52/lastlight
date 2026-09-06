use bevy::prelude::*;

/// Shared by camera and pointer projection.
pub const VIEW_HEIGHT: f32 = 390.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Title,
    Playing,
    Paused,
    Discovery,
    Dead,
    Ending,
}

#[derive(Resource)]
pub struct Game {
    pub test_mode: bool,
    pub mode: Mode,
    pub player: Vec2,
    pub velocity: Vec2,
    pub ground_speed: f32,
    pub facing: Vec2,
    pub brightness: f32,
    pub pulse: f32,
    pub pulse_cooldown: f32,
    pub anchor: Option<Vec2>,
    pub anchor_charge: f32,
    pub anchor_shell: Option<Vec2>,
    pub anchor_life: f32,
    pub anchor_placing: f32,
    pub anchor_origin: Vec2,
    pub anchor_snuff: f32,
    pub oil: u32,
    pub collected: u32,
    pub charted: u32,
    pub bell_found: bool,
    pub bell_out: Option<Vec2>,
    pub bell_origin: Vec2,
    pub bell_age: f32,
    pub has_anchor: bool,
    pub has_veil: bool,
    pub resonators: u32,
    pub secrets: u32,
    pub danger: f32,
    pub elapsed: f32,
    pub visited: Vec<bool>,
    pub checkpoint: Vec2,
    pub message: String,
    pub message_time: f32,
    pub deaths: u32,
    pub camera: Vec2,
    pub muted: bool,
    pub target: Option<Vec2>,
    pub room: usize,
    pub room_name_time: f32,
    pub invulnerable: f32,
    pub dark_time: f32,
    pub progress: f32,
    pub save_clock: f32,
    pub ending_time: f32,
    pub pulses: u32,
    pub moved: f32,
    pub gait: f32,
    pub impact: f32,
    pub impact_cooldown: f32,
    pub brace: f32,
    pub hurt_flash: f32,
    pub foot_echo: f32,
    pub idle: f32,
    pub region_seen: [bool; 4],
    pub body_facing: f32,
    pub hint_stage: u32,
    pub seed: u64,
    pub map_open: bool,
    pub started: bool,
    pub new_confirm: bool,
}
impl Game {
    pub fn new(seed: u64, spawn: Vec2, rooms: usize) -> Self {
        Self {
            test_mode: false,
            mode: Mode::Title,
            player: spawn,
            velocity: Vec2::ZERO,
            ground_speed: 0.,
            facing: Vec2::X,
            brightness: 0.55,
            pulse: 0.,
            pulse_cooldown: 0.,
            anchor: None,
            anchor_charge: 0.,
            anchor_shell: None,
            anchor_life: 0.,
            anchor_placing: 0.,
            anchor_origin: spawn,
            anchor_snuff: 0.,
            oil: 0,
            collected: 0,
            charted: 0,
            bell_found: false,
            bell_out: None,
            bell_origin: spawn,
            bell_age: 0.,
            has_anchor: false,
            has_veil: false,
            resonators: 0,
            secrets: 0,
            danger: 0.,
            elapsed: 0.,
            visited: vec![false; rooms],
            checkpoint: spawn,
            message: String::new(),
            message_time: 0.,
            deaths: 0,
            camera: spawn,
            muted: false,
            target: None,
            room: 0,
            room_name_time: 0.,
            invulnerable: 2.,
            dark_time: 0.,
            progress: 0.,
            save_clock: 0.,
            ending_time: 0.,
            pulses: 0,
            moved: 0.,
            gait: 0.,
            impact: 0.,
            impact_cooldown: 0.,
            brace: 0.,
            hurt_flash: 0.,
            foot_echo: 0.,
            idle: 1.,
            region_seen: [false; 4],
            body_facing: 1.,
            hint_stage: 0,
            seed,
            map_open: false,
            started: false,
            new_confirm: false,
        }
    }
    pub fn say(&mut self, text: impl Into<String>, duration: f32) {
        self.message = text.into();
        self.message_time = duration;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatureKind {
    Listener,
    Still,
    Grazer,
    Leviathan,
}
#[derive(Component)]
pub struct Creature {
    pub kind: CreatureKind,
    pub home: Vec2,
    pub position: Vec2,
    pub velocity: Vec2,
    pub alert: f32,
    pub phase: f32,
    pub room: usize,
    pub gait: f32,
    pub last_known: Vec2,
    pub sense_age: f32,
    pub notice: f32,
    pub facing: f32,
}

/// Static rock stops both the player and sight. Sampled at a conservative stride.
pub fn line_clear(world: &crate::world::WorldMap, a: Vec2, b: Vec2) -> bool {
    for gate in &world.gates {
        if gate.open > 0.82 || gate.latched {
            continue;
        }
        let da = (a - gate.position).dot(gate.normal);
        let db = (b - gate.position).dot(gate.normal);
        if da * db <= 0. && (da - db).abs() > 0.001 {
            let crossing = a.lerp(b, da / (da - db));
            if (crossing - gate.position).perp_dot(gate.normal).abs() < gate.half_width + 12. {
                return false;
            }
        }
    }
    let steps = (a.distance(b) / 22.).ceil() as u32;
    (1..steps).all(|i| world.field(a.lerp(b, i as f32 / steps as f32)) > 2.)
}
