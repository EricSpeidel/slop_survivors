mod game;

use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
// On WebAssembly, many static servers (including Trunk dev server) may serve HTML for unknown paths
// like "*.meta". Disable meta checks to avoid deserialization errors on web.
#[cfg(target_arch = "wasm32")]
use bevy::asset::AssetMetaCheck;
use game::GamePlugin;

fn main() {
    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    {
        app.insert_resource(AssetMetaCheck::Never);
    }

    app
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Slop Survivors".into(),
                // Logical resolution can be anything; on web we fit canvas to parent
                resolution: (1280.0, 720.0).into(),
                canvas: Some("#bevy".into()),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(GamePlugin)
        .run();
}
