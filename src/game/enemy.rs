use bevy::prelude::*;

use super::player::Player;
use super::states::GameState;

pub struct EnemyPlugin;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemyHealth {
    pub hp: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct EnemyHpBarFill; // child sprite scaled to hp ratio

#[derive(Component)]
pub struct EnemyHpBarRoot; // root to position over enemy

#[derive(Component, Deref, DerefMut)]
pub struct EnemySpeed(pub f32);

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            enemy_seek,
            update_enemy_hp_bars,
        ).run_if(in_state(GameState::Playing)));
    }
}

fn enemy_seek(
    mut enemies: Query<(&EnemySpeed, &mut Transform), With<Enemy>>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player.get_single() else { return; };
    for (speed, mut tf) in enemies.iter_mut() {
        let to_player = (player_tf.translation - tf.translation).truncate();
        if to_player.length_squared() > 0.1 {
            let dir = to_player.normalize();
            tf.translation.x += dir.x * **speed * time.delta_seconds();
            tf.translation.y += dir.y * **speed * time.delta_seconds();
        }
    }
}

fn update_enemy_hp_bars(
    enemies: Query<(&EnemyHealth, &Children), With<Enemy>>,
    mut roots: Query<(&Children, &mut Transform), (With<EnemyHpBarRoot>, Without<EnemyHpBarFill>)>,
    mut fills_tf: Query<&mut Transform, (With<EnemyHpBarFill>, Without<EnemyHpBarRoot>)>,
) {
    for (health, enemy_children) in enemies.iter() {
        let ratio = if health.max > 0.0 { (health.hp / health.max).clamp(0.0, 1.0) } else { 0.0 };
        // find bar root among enemy children
        for &child in enemy_children.iter() {
            if let Ok((bar_children, mut bar_root_tf)) = roots.get_mut(child) {
                bar_root_tf.translation.y = 20.0; // ensure stays above enemy
                for &bar_child in bar_children.iter() {
                    if let Ok(mut fill_tf) = fills_tf.get_mut(bar_child) {
                        let clamped = ratio.max(0.0);
                        fill_tf.scale.x = clamped.max(0.05);
                        // Anchor left edge: half width = 12. Center should sit at (-12 + half_width*scale.x*2)/2 simplifying to (-12.0 + 12.0*scale.x)
                        fill_tf.translation.x = -12.0 + 12.0 * fill_tf.scale.x;
                    }
                }
            }
        }
    }
}
