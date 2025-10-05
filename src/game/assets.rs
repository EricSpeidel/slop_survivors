use bevy::prelude::*;

use super::states::GameState;

pub struct AssetsPlugin;

#[derive(Resource, Default, Clone)]
pub struct EnemyAssets {
    pub bucket: Handle<Image>,
}

#[derive(Resource, Default, Clone)]
pub struct PlayerAssets {
    pub mainchar: Handle<Image>,
    pub flame: Handle<Image>,
}

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyAssets>()
            .init_resource::<PlayerAssets>()
            .add_systems(OnEnter(GameState::Playing), (load_enemy_assets, load_player_assets));
    }
}

fn load_enemy_assets(mut assets: ResMut<EnemyAssets>, asset_server: Res<AssetServer>) {
    // Correct path inside assets directory
    assets.bucket = asset_server.load("sprites/bucket.png");
}

fn load_player_assets(mut assets: ResMut<PlayerAssets>, asset_server: Res<AssetServer>) {
    assets.mainchar = asset_server.load("sprites/mainchar.png");
    assets.flame = asset_server.load("sprites/flame.png");
}
