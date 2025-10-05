use bevy::prelude::*;

use super::player::{Player, PlayerStats};
use super::states::GameState;

pub struct XpPlugin;

#[derive(Resource, Default)]
pub struct PendingLevelUps(pub u32);

#[derive(Component)]
pub struct XpOrb {
    pub value: u32,
}

pub fn spawn_xp_orb_at(commands: &mut Commands, pos: Vec2, value: u32) {
    commands.spawn((
        XpOrb { value },
        SpriteBundle {
            sprite: Sprite { color: Color::rgb(0.2, 1.0, 0.4), custom_size: Some(Vec2::splat(10.0)), ..default() },
            transform: Transform::from_xyz(pos.x, pos.y, 0.0),
            ..default()
        },
    ));
}

impl Plugin for XpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingLevelUps>()
            .add_systems(Update, (
                pickup_xp_orbs,
                enter_levelup_when_pending,
            ).run_if(in_state(GameState::Playing)));
    }
}

fn pickup_xp_orbs(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut stats: ResMut<PlayerStats>,
    mut pending: ResMut<PendingLevelUps>,
    orbs: Query<(Entity, &Transform, &XpOrb)>,
) {
    let Ok(player_tf) = player.get_single() else { return; };
    for (entity, tf, orb) in orbs.iter() {
        if player_tf.translation.truncate().distance(tf.translation.truncate()) < 28.0 {
            stats.xp += orb.value;
            // simple level formula: every 100 xp => level up & heal small amount
            let expected_level = stats.xp / 100 + 1;
            if expected_level > stats.level {
                let gained = expected_level - stats.level;
                stats.level = expected_level;
                stats.max_hp += 10.0 * gained as f32;
                stats.hp = stats.max_hp;
                pending.0 += gained; // queue selections
            }
            commands.entity(entity).despawn();
        }
    }
}

fn enter_levelup_when_pending(
    pending: Res<PendingLevelUps>,
    mut next: ResMut<NextState<GameState>>,
) {
    if pending.0 > 0 {
        next.set(GameState::LevelUp);
    }
}
