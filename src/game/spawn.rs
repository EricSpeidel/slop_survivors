use bevy::prelude::*;
use rand::Rng;

use super::enemy::{Enemy, EnemySpeed, EnemyHealth, EnemyHpBarFill, EnemyHpBarRoot};
use super::assets::EnemyAssets;
use super::states::GameState;

pub struct SpawnPlugin;

#[derive(Component)]
struct AwaitingTexture; // marker for enemies spawned with a placeholder

#[derive(Resource)]
struct EnemySpawnTimer {
    timer: Timer,
    elapsed: f32,
}

impl Default for EnemySpawnTimer {
    fn default() -> Self {
        Self { timer: Timer::from_seconds(1.2, TimerMode::Repeating), elapsed: 0.0 }
    }
}

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnTimer::default())
            .add_systems(Update, (
                spawn_enemies,
                upgrade_enemy_textures,
            ).run_if(in_state(GameState::Playing)));
    }
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut timer_res: ResMut<EnemySpawnTimer>,
    windows: Query<&Window>,
    enemy_assets: Res<EnemyAssets>,
    assets_images: Res<Assets<Image>>,
) {
    let Ok(primary) = windows.get_single() else { return; };
    // Increase difficulty over time: reduce interval gradually to a floor
    timer_res.elapsed += time.delta_seconds();
    let target = (1.2 - (timer_res.elapsed / 60.0)).clamp(0.25, 1.2); // after a minute approach 0.25s
    if (timer_res.timer.duration().as_secs_f32() - target).abs() > 0.05 {
        timer_res.timer.set_duration(std::time::Duration::from_secs_f32(target));
    }
    if timer_res.timer.tick(time.delta()).just_finished() {
    let mut rng = rand::rng();
    let side = rng.random_range(0..4);
        let w = primary.width();
        let h = primary.height();
        let (x, y) = match side {
            0 => (rng.random_range(-w..w), h + 50.0),
            1 => (rng.random_range(-w..w), -h - 50.0),
            2 => (w + 50.0, rng.random_range(-h..h)),
            _ => (-w - 50.0, rng.random_range(-h..h)),
        };
    let max_hp = rng.random_range(80.0..120.0);
        let use_texture = assets_images.get(&enemy_assets.bucket).is_some();
        let sprite_bundle = if use_texture {
            SpriteBundle {
                texture: enemy_assets.bucket.clone(),
                sprite: Sprite { color: Color::WHITE, custom_size: Some(Vec2::splat(32.0)), ..default() },
                transform: Transform::from_xyz(x, y, 5.0),
                ..default()
            }
        } else {
            // Visible placeholder (red quad) while texture is still loading on web
            SpriteBundle {
                sprite: Sprite { color: Color::rgb(0.9, 0.2, 0.2), custom_size: Some(Vec2::splat(32.0)), ..default() },
                transform: Transform::from_xyz(x, y, 5.0),
                ..default()
            }
        };
        let mut e = commands.spawn((
            Enemy,
            EnemySpeed(rng.random_range(60.0..120.0)),
            EnemyHealth { hp: max_hp, max: max_hp },
            sprite_bundle,
        ));
        if !use_texture {
            e.insert(AwaitingTexture);
        }
        e.with_children(|parent| {
            parent.spawn((EnemyHpBarRoot, SpriteBundle {
                sprite: Sprite { color: Color::rgb(0.15, 0.15, 0.15), custom_size: Some(Vec2::new(24.0, 4.0)), ..default() },
                transform: Transform::from_xyz(0.0, 20.0, 2.0),
                ..default()
            })).with_children(|root| {
                root.spawn((EnemyHpBarFill, SpriteBundle {
                    sprite: Sprite { color: Color::rgb(0.1, 0.9, 0.1), custom_size: Some(Vec2::new(24.0, 4.0)), ..default() },
                    // Start centered (we'll shift negatively as it shrinks to keep left edge anchored)
                    transform: Transform { translation: Vec3::new(0.0, 0.0, 0.1), scale: Vec3::new(1.0, 1.0, 1.0), ..default() },
                    ..default()
                }));
            });
        });
    }
}

fn upgrade_enemy_textures(
    mut commands: Commands,
    enemy_assets: Res<EnemyAssets>,
    assets_images: Res<Assets<Image>>,
    mut awaiting: Query<(Entity, Option<&mut Sprite>), (With<Enemy>, With<AwaitingTexture>)>,
) {
    // If texture not ready yet, skip
    if assets_images.get(&enemy_assets.bucket).is_none() { return; }
    for (e, sprite_opt) in awaiting.iter_mut() {
        // Insert texture handle and remove the marker; clear tint to white
        let mut ecmd = commands.entity(e);
        ecmd.insert(enemy_assets.bucket.clone());
        ecmd.remove::<AwaitingTexture>();
        if let Some(mut sprite) = sprite_opt {
            sprite.color = Color::WHITE;
        }
    }
}
