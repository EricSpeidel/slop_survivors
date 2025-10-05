use bevy::prelude::*;

use super::states::GameState;
 
use bevy::asset::AssetServer;
use bevy::window::PrimaryWindow;
use bevy::input::touch::{TouchInput, TouchPhase};

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerAura; // visual + radius indicator child
#[derive(Component)]
pub struct OrbitingFlame {
    pub angle: f32,
    pub radius: f32,
    pub speed: f32,
    pub contact_radius: f32,
    pub contact_damage: f32,
}

#[derive(Component, Deref, DerefMut)]
pub struct MoveSpeed(pub f32);

#[derive(Resource, Default)]
pub struct PlayerStats {
    pub xp: u32,
    pub level: u32,
    pub hp: f32,
    pub max_hp: f32,
}

impl PlayerStats {
    pub fn apply_damage(&mut self, dmg: f32) {
        if self.hp > 0.0 { self.hp = (self.hp - dmg).max(0.0); }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerStats>()
            .init_resource::<TouchState>()
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, (
                touch_capture_system,
                player_movement,
                animate_orbiting_flames,
                sync_flame_radius,
                reveal_flames_when_ready,
            ).run_if(in_state(GameState::Playing)));
    }
}
fn spawn_player(
    mut commands: Commands,
    mut stats: ResMut<PlayerStats>,
    existing: Query<Entity, With<Player>>, 
    asset_server: Res<AssetServer>,
) {
    if existing.iter().next().is_some() {
        // Already have a player (likely returning from Pause) -> do not respawn or reset stats
        return;
    }
    // Initialize baseline stats only if truly starting fresh
    stats.level = 1;
    stats.max_hp = 100.0;
    stats.hp = stats.max_hp;
    stats.xp = 0;

    let mut player = commands.spawn((
        Player,
        MoveSpeed(350.0),
        SpatialBundle::default(),
    ));
    player.with_children(|parent| {
        // Player sprite: start white while texture loads; use a modest size like 48x48
        parent.spawn(SpriteBundle {
            texture: asset_server.load("sprites/mainchar.png"),
            sprite: Sprite { color: Color::WHITE, custom_size: Some(Vec2::splat(48.0)), ..default() },
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        });
        // Orbiting flames replacing the aura quad
        let flame_count = 6;
        let radius = 120.0; // match combat aura radius for visual consistency
        for i in 0..flame_count {
            let angle = i as f32 / flame_count as f32 * std::f32::consts::TAU;
            parent.spawn((PlayerAura, OrbitingFlame { angle, radius, speed: 1.8, contact_radius: 16.0, contact_damage: 5.0 }, SpriteBundle {
                texture: asset_server.load("sprites/flame.png"),
                // Start fully transparent to avoid untextured colored quads before texture finishes loading
                sprite: Sprite { color: Color::rgba(1.0, 1.0, 1.0, 0.0), custom_size: Some(Vec2::splat(32.0)), ..default() },
                transform: Transform::from_xyz(angle.cos() * radius, angle.sin() * radius, 0.0),
                ..default()
            }));
        }
    });
    commands.spawn(Camera2dBundle::default());
}

fn player_movement(
    kb: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut q: Query<(&MoveSpeed, &mut Transform), With<Player>>,
    time: Res<Time>,
    touch: Res<TouchState>,
) {
    if let Some((speed, mut tf)) = q.iter_mut().next() {
        // 1) Pointer follow: prefer active touch if present, else mouse cursor
        let mut moved_by_pointer = false;
        if let Ok(window) = windows.get_single() {
            let screen_pos_logical = if touch.active { touch.position } else { window.cursor_position() };
            if let Some(mut cursor_pos) = screen_pos_logical {
                // Bevy 0.13 camera.viewport_to_world_2d expects PHYSICAL pixel coords; convert from logical using scale factor
                let sf = window.resolution.scale_factor() as f32;
                cursor_pos *= sf;
                if let Ok((camera, cam_tf)) = camera_q.get_single() {
                    if let Some(world_pos) = camera.viewport_to_world_2d(cam_tf, cursor_pos) {
                        let player_pos = tf.translation.truncate();
                        let to_target = world_pos - player_pos;
                        if to_target.length_squared() > 1.0 { // small deadzone
                            let dir = to_target.normalize();
                            tf.translation.x += dir.x * **speed * time.delta_seconds();
                            tf.translation.y += dir.y * **speed * time.delta_seconds();
                            moved_by_pointer = true;
                        }
                    }
                }
            }
        }

        // 2) Fallback to keyboard if pointer didn't move us
        if !moved_by_pointer {
            let mut dir = Vec2::ZERO;
            if kb.pressed(KeyCode::KeyW) { dir.y += 1.0; }
            if kb.pressed(KeyCode::KeyS) { dir.y -= 1.0; }
            if kb.pressed(KeyCode::KeyA) { dir.x -= 1.0; }
            if kb.pressed(KeyCode::KeyD) { dir.x += 1.0; }
            if dir.length_squared() > 0.0 { dir = dir.normalize(); }
            tf.translation.x += dir.x * **speed * time.delta_seconds();
            tf.translation.y += dir.y * **speed * time.delta_seconds();
        }
    }
}

// Track current touch state (screen-space position and active flag)
#[derive(Resource, Default)]
struct TouchState {
    position: Option<Vec2>,
    active: bool,
}

fn touch_capture_system(
    mut ev_touch: EventReader<TouchInput>,
    mut state: ResMut<TouchState>,
) {
    for ev in ev_touch.read() {
        match ev.phase {
            TouchPhase::Started | TouchPhase::Moved => {
                state.position = Some(ev.position);
                state.active = true;
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                state.position = None;
                state.active = false;
            }
        }
    }
}

fn animate_orbiting_flames(
    time: Res<Time>,
    mut flames: Query<(&mut OrbitingFlame, &mut Transform), With<PlayerAura>>,
) {
    for (mut of, mut tf) in flames.iter_mut() {
        of.angle = (of.angle + of.speed * time.delta_seconds()) % std::f32::consts::TAU;
        let pos = Vec2::new(of.angle.cos() * of.radius, of.angle.sin() * of.radius);
        tf.translation.x = pos.x;
        tf.translation.y = pos.y;
        // Optional: small pulsing scale to simulate dancing flame
        let pulse = 0.9 + 0.1 * (of.angle * 3.0).sin();
        tf.scale = Vec3::splat(pulse);
        // Face outward (rotate around Z to tangent)—a simple spin looks nice
        tf.rotation = Quat::from_rotation_z(of.angle + std::f32::consts::FRAC_PI_2);
    }
}

// Keep flame radius synced to aura radius for a cohesive look
use crate::game::combat::AuraConfig;
fn sync_flame_radius(cfg: Res<AuraConfig>, mut flames: Query<&mut OrbitingFlame, With<PlayerAura>>) {
    if !cfg.is_changed() { return; }
    for mut of in flames.iter_mut() {
        of.radius = cfg.radius;
    }
}

fn reveal_flames_when_ready(
    images: Res<Assets<Image>>,
    mut q: Query<(&Handle<Image>, &mut Sprite), With<PlayerAura>>,
) {
    // Once any flame texture is loaded, ensure it becomes visible (white tint, full alpha)
    for (handle, mut sprite) in q.iter_mut() {
        if images.get(handle).is_some() {
            sprite.color = Color::rgba(1.0, 1.0, 1.0, 1.0);
        }
    }
}


// (Removed texture upgrade system; not needed as we tint white by default and handle loads asynchronously.)
