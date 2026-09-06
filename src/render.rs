//! Painted animation atlases and terrain share one distance-field lighting pass.
//! A bounded world pass keeps Retina/browser zoom from multiplying lighting cost.
use bevy::{
    asset::RenderAssetUsages,
    camera::{RenderTarget, visibility::RenderLayers},
    image::ImageSampler,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat, TextureUsages,
    },
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
    window::PrimaryWindow,
};

use crate::{model::*, world::*};

const MAX_ROOMS: usize = 32;
const MAX_OBJECTS: usize = 24;

/// Pose selection and lamp registration share a single clock. Distances of
/// 0 and 38 are the two heel contacts, also used by the recorded footsteps.
fn traveller_pose(game: &Game) -> Vec4 {
    use crate::traveller_lamps::{REST_LAMPS, WALK_LAMPS};
    let view = if game.facing.y.abs() > game.facing.x.abs() * 1.2 {
        if game.facing.y > 0. { 1 } else { 2 }
    } else {
        0
    };
    if game.ground_speed < 8. && game.idle > 0.15 {
        let landing = (game.gait / 38.).ceil() as usize % 2;
        let cycle = view * 2 + landing;
        let pose = (((game.idle - 0.15) / 0.85).clamp(0., 1.) * 23.) as usize;
        let lamp = REST_LAMPS[cycle][pose];
        Vec4::new(lamp[0], lamp[1], (cycle * 24 + pose) as f32, 1.)
    } else {
        let pose = (game.gait / 76. * 48.).floor() as usize % 48;
        let lamp = WALK_LAMPS[view][pose];
        Vec4::new(
            lamp[0],
            lamp[1],
            (pose + if view == 2 { 48 } else { 0 }) as f32,
            0.,
        )
    }
}

pub struct RenderPlugin;

#[cfg(test)]
mod traveller_tests {
    use super::*;

    #[test]
    fn walking_lantern_registration_follows_every_direction_and_contact() {
        let mut game = Game::new(72419, Vec2::ZERO, 32);
        game.ground_speed = 82.;
        game.idle = 0.;
        for (view, facing) in [Vec2::X, Vec2::Y, Vec2::NEG_Y].into_iter().enumerate() {
            game.facing = facing;
            for pose in 0..48 {
                game.gait = (pose as f32 + 0.25) * 76. / 48.;
                let sample = traveller_pose(&game);
                let expected = crate::traveller_lamps::WALK_LAMPS[view][pose];
                assert_eq!(sample.truncate().truncate(), Vec2::from_array(expected));
                assert_eq!(sample.z as usize, pose + if view == 2 { 48 } else { 0 });
                assert_eq!(sample.w, 0.);
            }
        }
        game.facing = Vec2::X;
        let right = traveller_pose(&game);
        game.facing = Vec2::NEG_X;
        game.body_facing = -1.;
        assert_eq!(
            traveller_pose(&game),
            right,
            "Left uses the identical whole-body pose, mirrored once in the shader"
        );
    }

    #[test]
    fn either_foot_settles_into_the_same_resting_body_and_lamp() {
        let mut game = Game::new(72419, Vec2::ZERO, 32);
        game.ground_speed = 0.;
        game.idle = 1.;
        for (view, facing) in [Vec2::X, Vec2::Y, Vec2::NEG_Y].into_iter().enumerate() {
            game.facing = facing;
            let mut last_lamp = Vec2::ZERO;
            for landing in 0..2 {
                game.gait = landing as f32 * 38.;
                let sample = traveller_pose(&game);
                assert_eq!(sample.z as usize, (view * 2 + landing) * 24 + 23);
                assert_eq!(sample.w, 1.);
                let lamp = sample.truncate().truncate();
                if landing == 1 {
                    assert_eq!(lamp, last_lamp);
                }
                last_lamp = lamp;
            }
        }
    }
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneLoading>()
            .add_plugins(Material2dPlugin::<CavernMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(PostUpdate, (check_scene_loading, update_view));
    }
}

#[derive(Clone, Debug, ShaderType)]
struct CavernUniform {
    view: Vec4,
    player: Vec4,
    anchor: Vec4,
    clock: Vec4,
    bounds: Vec4,
    counts: Vec4,
    controls: Vec4,
    motion: Vec4,
    appearance: Vec4,
    feedback: Vec4,
    journey: Vec4,
    tools: Vec4,
    placement: Vec4,
    bell: Vec4,
    traveller: Vec4,
    ritual: Vec4,
    finds: [Vec4; 16],
    roots: [Vec4; 108],
    root_widths: [Vec4; 108],
    tendrils: [Vec4; crate::shadow::SEGMENTS],
    props: [Vec4; 32],
    rooms: [Vec4; MAX_ROOMS],
    room_state: [Vec4; MAX_ROOMS],
    gates: [Vec4; MAX_OBJECTS],
    gate_state: [Vec4; MAX_OBJECTS],
    sites: [Vec4; MAX_OBJECTS],
    creatures: [Vec4; MAX_OBJECTS],
    creature_state: [Vec4; MAX_OBJECTS],
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CavernMaterial {
    #[uniform(0)]
    scene: CavernUniform,
    #[texture(1)]
    #[sampler(2)]
    field: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    ground: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    traveller: Handle<Image>,
    #[texture(7)]
    #[sampler(8)]
    hauler: Handle<Image>,
    #[texture(9)]
    #[sampler(10)]
    terrain: Handle<Image>,
    #[texture(11)]
    #[sampler(12)]
    ecosystem: Handle<Image>,
    #[texture(13)]
    #[sampler(14)]
    traveller_vertical: Handle<Image>,
    #[texture(15)]
    #[sampler(16)]
    traveller_idle: Handle<Image>,
    #[texture(17)]
    #[sampler(18)]
    objects: Handle<Image>,
}
impl Material2d for CavernMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/world.wgsl".into()
    }
}

#[derive(Resource)]
struct SceneAssets(Vec<bevy::asset::UntypedHandle>);

fn check_scene_loading(
    server: Res<AssetServer>,
    assets: Res<SceneAssets>,
    mut loading: ResMut<SceneLoading>,
) {
    if loading.ready || loading.failed {
        return;
    }
    loading.failed = assets.0.iter().any(|h| {
        matches!(
            server.get_load_state(h.id()),
            Some(bevy::asset::LoadState::Failed(_))
        )
    });
    loading.ready = assets
        .0
        .iter()
        .all(|h| server.is_loaded_with_dependencies(h.id()));
}

#[derive(Resource)]
struct CavernCanvas(Handle<CavernMaterial>, u64);
#[derive(Component)]
struct Canvas;
#[derive(Component)]
struct Presentation;
#[derive(Resource)]
struct WorldTarget(Handle<Image>);

fn rasterize(map: &WorldMap) -> (Image, Vec4) {
    let mut lo = Vec2::splat(f32::MAX);
    let mut hi = Vec2::splat(f32::MIN);
    for room in &map.rooms {
        lo = lo.min(room.center - room.radius - Vec2::splat(128.));
        hi = hi.max(room.center + room.radius + Vec2::splat(128.));
    }
    let size = hi - lo;
    // Eight world units per texel. Linear distance sampling provides a smooth
    // silhouette without a high-resolution texture or visible tile boundaries.
    let width = (size.x / 8.).ceil().clamp(64., 1280.) as u32;
    let height = (size.y / 8.).ceil().clamp(64., 1280.) as u32;
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let p = lo
                + size
                    * Vec2::new(
                        (x as f32 + 0.5) / width as f32,
                        (y as f32 + 0.5) / height as f32,
                    );
            let field = map.field(p);
            let room = map.room_at(p).min(map.rooms.len() - 1);
            let encoded = ((field.clamp(-256., 256.) + 256.) / 512. * 255.).round() as u8;
            pixels.extend_from_slice(&[encoded, map.rooms[room].region as u8, room as u8, 255]);
        }
    }
    let mut field = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    field.sampler = ImageSampler::linear();
    (field, Vec4::new(lo.x, lo.y, size.x, size.y))
}

#[allow(clippy::too_many_arguments)]
fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    map: Res<WorldMap>,
    game: Res<Game>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CavernMaterial>>,
) {
    let (field, bounds) = rasterize(&map);
    let scene = CavernUniform {
        view: Vec4::new(game.player.x, game.player.y, 1200., VIEW_HEIGHT),
        player: Vec4::new(game.player.x, game.player.y, 0.35, 0.),
        anchor: Vec4::ZERO,
        clock: Vec4::ZERO,
        bounds,
        counts: Vec4::ZERO,
        controls: Vec4::ZERO,
        motion: Vec4::ZERO,
        feedback: Vec4::ZERO,
        journey: Vec4::ZERO,
        tools: Vec4::ZERO,
        placement: Vec4::ZERO,
        bell: Vec4::ZERO,
        traveller: traveller_pose(&game),
        ritual: Vec4::ZERO,
        finds: [Vec4::ZERO; 16],
        roots: [Vec4::ZERO; 108],
        root_widths: [Vec4::ZERO; 108],
        tendrils: [Vec4::ZERO; crate::shadow::SEGMENTS],
        appearance: Vec4::ZERO,
        props: [Vec4::ZERO; 32],
        rooms: [Vec4::ZERO; MAX_ROOMS],
        room_state: [Vec4::ZERO; MAX_ROOMS],
        gates: [Vec4::ZERO; MAX_OBJECTS],
        gate_state: [Vec4::ZERO; MAX_OBJECTS],
        sites: [Vec4::ZERO; MAX_OBJECTS],
        creatures: [Vec4::ZERO; MAX_OBJECTS],
        creature_state: [Vec4::ZERO; MAX_OBJECTS],
    };
    let material = materials.add(CavernMaterial {
        scene,
        field: images.add(field),
        ground: server.load("art/surfaces.png"),
        traveller: server.load("art/traveller-motion.png"),
        hauler: server.load("art/hauler-motion.png"),
        terrain: server.load("art/terrain.png"),
        ecosystem: server.load("art/ecosystem-motion.png"),
        traveller_vertical: server.load("art/traveller-vertical-motion.png"),
        traveller_idle: server.load("art/traveller-idle.png"),
        objects: server.load("art/objects.png"),
    });
    let art = materials
        .get(&material)
        .expect("Just created scene material");
    commands.insert_resource(SceneAssets(vec![
        art.ground.clone().untyped(),
        art.traveller.clone().untyped(),
        art.hauler.clone().untyped(),
        art.terrain.clone().untyped(),
        art.ecosystem.clone().untyped(),
        art.traveller_vertical.clone().untyped(),
        art.traveller_idle.clone().untyped(),
        art.objects.clone().untyped(),
        server
            .load::<bevy::shader::Shader>("shaders/world.wgsl")
            .untyped(),
    ]));
    commands.insert_resource(CavernCanvas(material.clone(), game.seed));
    let mut target = Image::new_target_texture(1280, 800, TextureFormat::Rgba8UnormSrgb, None);
    target.texture_descriptor.usage |= TextureUsages::TEXTURE_BINDING;
    target.sampler = ImageSampler::linear();
    let target = images.add(target);
    commands.insert_resource(WorldTarget(target.clone()));
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            ..default()
        },
        RenderTarget::Image(target.clone().into()),
        RenderLayers::layer(1),
        Msaa::Off,
    ));
    commands.spawn((Camera2d, Msaa::Off, IsDefaultUiCamera));
    commands.spawn((Sprite::from_image(target), Presentation));
    commands.spawn((
        Canvas,
        RenderLayers::layer(1),
        Mesh2d(meshes.add(Rectangle::new(1., 1.))),
        MeshMaterial2d(material),
        Transform::from_scale(Vec3::new(1200., 760., 1.)),
    ));
}

// Bevy injects these independently borrowed resources and queries as system
// parameters; grouping them merely to reduce argument count obscures access.
#[allow(clippy::too_many_arguments)]
fn update_view(
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut game: ResMut<Game>,
    map: Res<WorldMap>,
    creatures: Query<&Creature>,
    roots: Res<crate::roots::Rootwork>,
    canvas: Option<ResMut<CavernCanvas>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<CavernMaterial>>,
    mut transform: Query<&mut Transform, With<Canvas>>,
    target: Res<WorldTarget>,
    mut presentation: Query<&mut Sprite, With<Presentation>>,
) {
    let (Ok(window), Some(mut canvas)) = (windows.single(), canvas) else {
        return;
    };
    let Some(material) = materials.get_mut(&canvas.0) else {
        return;
    };
    if canvas.1 != game.seed {
        let (field, bounds) = rasterize(&map);
        material.field = images.add(field);
        material.scene.bounds = bounds;
        canvas.1 = game.seed;
    }
    let dt = time.delta_secs().min(0.1);
    let lead = if game.mode == Mode::Title {
        Vec2::new(-95., 5.)
    } else {
        game.facing * 16. + game.velocity * 0.12
    };
    let desired = game.player + lead;
    let camera = game.camera;
    game.camera += (desired - camera) * (1. - (-dt * 3.4).exp());
    let width = window.width().max(1.);
    let height = window.height().max(1.);
    let scale = (1600. / width.max(height)).min(window.scale_factor().min(1.25));
    let size = Extent3d {
        width: (width * scale).round().max(1.) as u32,
        height: (height * scale).round().max(1.) as u32,
        depth_or_array_layers: 1,
    };
    if images
        .get(&target.0)
        .is_some_and(|image| image.texture_descriptor.size != size)
        && let Some(image) = images.get_mut(&target.0)
    {
        image.resize(size);
    }
    for mut sprite in &mut presentation {
        sprite.custom_size = Some(Vec2::new(width, height));
    }
    for mut transform in &mut transform {
        transform.scale = Vec3::new(size.width as f32, size.height as f32, 1.);
    }
    let scene = &mut material.scene;
    scene.view = Vec4::new(
        game.camera.x + (time.elapsed_secs() * 71.).sin() * game.hurt_flash * 1.7,
        game.camera.y
            + game.impact * 0.18
            + (time.elapsed_secs() * 59.).cos() * game.hurt_flash * 1.1,
        VIEW_HEIGHT * width / height,
        VIEW_HEIGHT,
    );
    if game.map_open {
        let map_height =
            ((scene.bounds.w + 100.) / 0.75).max((scene.bounds.z + 100.) * height / width);
        scene.view = Vec4::new(
            scene.bounds.x + scene.bounds.z * 0.5,
            scene.bounds.y + scene.bounds.w * 0.5 - map_height * 0.09,
            map_height * width / height,
            map_height,
        );
    }
    scene.appearance = Vec4::new(game.facing.x, game.facing.y, game.ground_speed, 0.);
    scene.traveller = traveller_pose(&game);
    scene.ritual = crate::ritual::visual(&game, &map);
    scene.feedback = Vec4::new(game.hurt_flash, game.brace, game.foot_echo, game.idle);
    scene.journey = Vec4::new(
        game.resonators as f32,
        f32::from(game.has_anchor),
        f32::from(game.has_veil),
        0.,
    );
    scene.placement = game.anchor_origin.extend(0.).extend(0.);
    scene.tools = Vec4::new(
        game.anchor_life,
        game.anchor_placing,
        game.anchor_snuff,
        game.charted as f32,
    );
    scene.bell = game
        .bell_out
        .map(|p| {
            let t = (game.bell_age / 0.65).min(1.);
            let p = game.bell_origin.lerp(p, t) + Vec2::Y * (t * std::f32::consts::PI).sin() * 35.;
            Vec4::new(p.x, p.y, 1., game.bell_age)
        })
        .unwrap_or(Vec4::ZERO);
    scene.motion = Vec4::new(game.gait, game.body_facing, game.impact, 0.);
    scene.player = Vec4::new(game.player.x, game.player.y, game.brightness, game.pulse);
    scene.anchor = game
        .anchor
        .or(game.anchor_shell)
        .map(|p| Vec4::new(p.x, p.y, f32::from(game.anchor.is_some()), 1.))
        .unwrap_or(Vec4::ZERO);
    scene.clock = Vec4::new(
        time.elapsed_secs(),
        game.danger,
        game.secrets as f32,
        game.pulse_cooldown,
    );
    scene.controls = Vec4::new(
        if game.map_open { 1. } else { 0. },
        game.progress,
        if game.mode == Mode::Dead { 1. } else { 0. },
        if game.mode == Mode::Ending {
            game.ending_time
        } else {
            0.
        },
    );
    scene.counts.x = map.rooms.len().min(MAX_ROOMS) as f32;
    for (i, room) in map.rooms.iter().take(MAX_ROOMS).enumerate() {
        scene.rooms[i] = Vec4::new(room.center.x, room.center.y, room.radius.x, room.radius.y);
        scene.room_state[i] = Vec4::new(
            room.region as f32,
            if game.visited.get(i).copied().unwrap_or(false) {
                1.
            } else {
                0.
            },
            room.id as f32,
            0.,
        );
    }
    // Cull CPU-side before filling a small fixed uniform array. This also keeps
    // shader loop counts low when exploring a much larger connected world.
    let cull_radius = scene.view.z.max(scene.view.w) * 0.62 + 250.;
    let cull_center = Vec2::new(scene.view.x, scene.view.y);
    scene.finds.fill(Vec4::ZERO);
    for (i, find) in crate::exploration::finds(&map)
        .iter()
        .filter(|f| f.position.distance(cull_center) < cull_radius)
        .take(16)
        .enumerate()
    {
        let kind = match find.kind {
            crate::exploration::FindKind::Bell => 1.,
            crate::exploration::FindKind::Oil => 2.,
            crate::exploration::FindKind::Route => 3.,
        };
        let collected = game.collected & (1 << find.id) != 0;
        scene.finds[i] = Vec4::new(
            find.position.x,
            find.position.y,
            if collected && kind < 2.5 { 0. } else { kind },
            f32::from(collected),
        );
    }
    scene.roots.fill(Vec4::ZERO);
    scene.root_widths.fill(Vec4::ZERO);
    let mut patches: Vec<_> = roots
        .patches
        .iter()
        .filter(|r| r.center.distance(cull_center) < cull_radius)
        .collect();
    patches.sort_by(|a, b| {
        a.center
            .distance_squared(cull_center)
            .total_cmp(&b.center.distance_squared(cull_center))
    });
    let mut root_count = 0;
    for patch in patches {
        for seg in patch.visible_segments() {
            if root_count >= 108 {
                break;
            }
            scene.roots[root_count] = Vec4::new(seg.a.x, seg.a.y, seg.b.x, seg.b.y);
            scene.root_widths[root_count] = Vec4::new(seg.ra, seg.rb, 0., 0.);
            root_count += 1;
        }
    }
    scene.appearance.w = root_count as f32;
    // Objects belong to physical ledges. A room gets at most one masonry arch;
    // everyday rubble uses low, distinct silhouettes instead of resized arches.
    let choose = |region: u32, id: usize| -> usize {
        match region {
            1 => [9, 10, 0, 6, 9, 10][id % 6],
            2 => [5, 6, 8, 11, 5, 2][id % 6],
            3 => [6, 11, 2, 10, 6][id % 5],
            _ => [6, 10, 2, 9, 6][id % 5],
        }
    };
    let mut props = Vec::new();
    for room in &map.rooms {
        for (index, rock) in room.formations.iter().enumerate() {
            let position = (rock.a + rock.b) * 0.5;
            if position.distance(cull_center) < cull_radius {
                let kind = if room.region == 2 && index == 0 && room.id % 2 == 1 {
                    1
                } else {
                    choose(room.region, room.id * 7 + index)
                };
                props.push(Vec4::new(
                    position.x,
                    position.y,
                    58. + rock.radius * 1.15,
                    kind as f32 + if (room.id + index) % 2 == 0 { 0.25 } else { 0. },
                ));
            }
        }
        for (index, edge) in room.boundary.iter().enumerate() {
            let position = *edge + (*edge - room.center).normalize_or_zero() * 15.;
            if index % 3 == 0
                && position.distance(cull_center) < cull_radius
                && map.field(position) < -2.
            {
                if props
                    .iter()
                    .any(|p: &Vec4| Vec2::new(p.x, p.y).distance(position) < (p.z + 100.) * 0.43)
                {
                    continue;
                }
                let kind = choose(room.region, room.id * 3 + index / 3);
                props.push(Vec4::new(
                    position.x,
                    position.y,
                    82. + (index % 4) as f32 * 11.,
                    kind as f32 + if (room.id + index) % 2 == 0 { 0.25 } else { 0. },
                ));
            }
        }
    }
    props.sort_by(|a, b| {
        a.truncate()
            .truncate()
            .distance_squared(cull_center)
            .total_cmp(&b.truncate().truncate().distance_squared(cull_center))
    });
    props.truncate(32);
    props.sort_by(|a, b| b.y.total_cmp(&a.y));
    scene.motion.w = props.len() as f32;
    for (index, prop) in props.into_iter().enumerate() {
        scene.props[index] = prop;
    }
    let mut n = 0;
    for gate in &map.gates {
        if gate.position.distance(cull_center) > cull_radius || n == MAX_OBJECTS {
            continue;
        }
        scene.gates[n] = Vec4::new(
            gate.position.x,
            gate.position.y,
            gate.normal.x,
            gate.normal.y,
        );
        let kind = match gate.kind {
            GateKind::Light => 0.,
            GateKind::Shade => 1.,
            GateKind::Anchor => 2.,
            GateKind::Veil => 3.,
            GateKind::Final => 4.,
        };
        scene.gate_state[n] = Vec4::new(
            gate.half_width,
            gate.open,
            kind,
            if gate.latched { 1. } else { 0. },
        );
        n += 1;
    }
    scene.counts.y = n as f32;
    n = 0;
    for site in &map.sites {
        if site.position.distance(cull_center) > cull_radius || n == MAX_OBJECTS {
            continue;
        }
        let kind = match site.kind {
            SiteKind::Sanctuary => 0.,
            SiteKind::AnchorAbility => 1.,
            SiteKind::VeilAbility => 2.,
            SiteKind::Resonator => {
                if site.room == 16 {
                    3.2
                } else {
                    3.
                }
            }
            SiteKind::Secret => 4.,
            SiteKind::Ending => 5.,
        };
        scene.sites[n] = Vec4::new(
            site.position.x,
            site.position.y,
            kind,
            if site.active { 1. } else { 0. },
        );
        n += 1;
    }
    scene.counts.z = n as f32;
    n = 0;
    for creature in &creatures {
        if creature.kind == CreatureKind::Leviathan {
            scene.tendrils = crate::shadow::branches(creature);
            scene.journey.w = if creature.position.distance(cull_center) < cull_radius + 650. {
                1.
            } else {
                0.
            };
            continue;
        }
        if creature.position.distance(cull_center) > cull_radius + 350. || n == MAX_OBJECTS {
            continue;
        }
        let kind = match creature.kind {
            CreatureKind::Listener => 0.,
            CreatureKind::Still => 1.,
            CreatureKind::Grazer => 2.,
            CreatureKind::Leviathan => 3.,
        };
        scene.creatures[n] = Vec4::new(
            creature.position.x,
            creature.position.y,
            kind,
            creature.alert,
        );
        scene.creature_state[n] = Vec4::new(
            creature.velocity.x,
            creature.velocity.y,
            creature.gait,
            creature.facing,
        );
        n += 1;
    }
    scene.counts.w = n as f32;
}
