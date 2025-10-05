use bevy::prelude::*;

use super::enemy::{Enemy, EnemyHealth};
use super::player::{Player, PlayerStats};
use super::xp::spawn_xp_orb_at;
use super::states::GameState;
use super::player::OrbitingFlame;

pub struct CombatPlugin;

// Constants for combat tuning
const COLLISION_DISTANCE: f32 = 28.0;
const COLLISION_PLAYER_DAMAGE: f32 = 5.0;
const COLLISION_ENEMY_DAMAGE: f32 = 20.0;
const AURA_TICK_SECONDS: f32 = 0.25; // damage application interval (faster than once per second)

#[derive(Resource)]
pub struct AuraConfig {
    pub radius: f32,
    pub damage_tick: f32,
}

impl Default for AuraConfig {
    fn default() -> Self { Self { radius: 120.0, damage_tick: 10.0 } }
}

#[derive(Resource)]
pub struct AuraTickTimer(Timer);

#[derive(Component)]
pub struct Damage(pub f32);

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AuraTickTimer>()
            .init_resource::<AuraConfig>()
            .add_systems(Update, (
                collision_combat,
                aura_tick_damage,
                flames_contact_damage,
            ).run_if(in_state(GameState::Playing)));
    }
}

impl Default for AuraTickTimer {
    fn default() -> Self { Self(Timer::from_seconds(AURA_TICK_SECONDS, TimerMode::Repeating)) }
}

impl AuraTickTimer {
    pub fn duration_secs(&self) -> f32 { self.0.duration().as_secs_f32() }
    pub fn set_duration_secs(&mut self, secs: f32) { self.0.set_duration(std::time::Duration::from_secs_f32(secs)); }
}

fn collision_combat(
    mut commands: Commands,
    mut stats: ResMut<PlayerStats>,
    players: Query<&Transform, With<Player>>,
    mut enemies: Query<(Entity, &Transform, &mut EnemyHealth), With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(player_tf) = players.get_single() else { return; };
    let mut took_damage = false;
    for (enemy_entity, tf, mut eh) in enemies.iter_mut() {
        let dist = player_tf.translation.truncate().distance(tf.translation.truncate());
        if dist < COLLISION_DISTANCE {
            stats.apply_damage(COLLISION_PLAYER_DAMAGE);
            took_damage = true;
            eh.hp -= COLLISION_ENEMY_DAMAGE;
        }
        if eh.hp <= 0.0 {
            spawn_xp_orb_at(&mut commands, tf.translation.truncate(), 5);
            commands.entity(enemy_entity).despawn_recursive();
        }
    }
    if took_damage && stats.hp <= 0.0 {
        next_state.set(GameState::GameOver);
    }
}

// Flames do contact damage on touch in addition to the periodic aura damage
fn flames_contact_damage(
    mut commands: Commands,
    flames: Query<&GlobalTransform, With<OrbitingFlame>>,
    mut enemies: Query<(Entity, &Transform, &mut EnemyHealth), With<Enemy>>,
) {
    // Build a list of flame world positions once
    let mut flame_positions: Vec<Vec2> = Vec::new();
    for tf in flames.iter() {
        let pos = tf.compute_transform().translation.truncate();
        flame_positions.push(pos);
    }
    for (entity, tf, mut eh) in enemies.iter_mut() {
        let epos = tf.translation.truncate();
        for fpos in flame_positions.iter() {
            if epos.distance(*fpos) < 16.0 {
                eh.hp -= 5.0;
                if eh.hp <= 0.0 {
                    spawn_xp_orb_at(&mut commands, tf.translation.truncate(), 5);
                    commands.entity(entity).despawn_recursive();
                }
                break;
            }
        }
    }
}

fn aura_tick_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<AuraTickTimer>,
    player: Query<&Transform, With<Player>>, // use player world transform (child aura was local)
    mut enemies: Query<(Entity, &Transform, &mut EnemyHealth), With<Enemy>>,
    cfg: Res<AuraConfig>,
) {
    if !timer.0.tick(time.delta()).just_finished() { return; }
    let Ok(player_tf) = player.get_single() else { return; };
    let center = player_tf.translation.truncate();
    let radius_sq = cfg.radius * cfg.radius;
    for (entity, tf, mut eh) in enemies.iter_mut() {
        let dist_sq = center.distance_squared(tf.translation.truncate());
        if dist_sq < radius_sq {
            eh.hp -= cfg.damage_tick;
            if eh.hp <= 0.0 {
                spawn_xp_orb_at(&mut commands, tf.translation.truncate(), 5);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
