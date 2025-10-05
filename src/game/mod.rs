pub mod states;
pub mod player;
pub mod enemy;
pub mod spawn;
pub mod movement;
pub mod xp;
pub mod ui;
pub mod assets;
pub mod combat;

use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use bevy::window::{PrimaryWindow, WindowMode};
#[cfg(target_arch = "wasm32")]
use web_sys::window as web_window;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use states::*;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                assets::AssetsPlugin,
                player::PlayerPlugin,
                enemy::EnemyPlugin,
                spawn::SpawnPlugin,
                movement::MovementPlugin,
                combat::CombatPlugin,
                xp::XpPlugin,
                ui::UiPlugin,
            ));
        // Input driven state toggles
        app.add_systems(Update, (
            toggle_pause,
            restart_game,
            auto_start_loading,
            #[cfg(target_arch = "wasm32")] resize_canvas_to_window,
            #[cfg(target_arch = "wasm32")] toggle_fullscreen,
        ));
    }
}

fn toggle_pause(kb: Res<ButtonInput<KeyCode>>, state: Res<State<GameState>>, mut next: ResMut<NextState<GameState>>) {
    if kb.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Playing => next.set(GameState::Paused),
            GameState::Paused => next.set(GameState::Playing),
            _ => {}
        }
    }
}

fn restart_game(kb: Res<ButtonInput<KeyCode>>, state: Res<State<GameState>>, mut next: ResMut<NextState<GameState>>, mut stats: ResMut<player::PlayerStats>, mut q_player: Query<&mut Transform, With<player::Player>>) {
    if kb.just_pressed(KeyCode::KeyR) && matches!(state.get(), GameState::GameOver) {
        // Reset player stats & position
        *stats = player::PlayerStats { level: 1, max_hp: 100.0, hp: 100.0, xp: 0 };
        if let Some(mut tf) = q_player.iter_mut().next() { tf.translation = Vec3::ZERO; }
        next.set(GameState::Playing);
    }
}

fn auto_start_loading(state: Res<State<GameState>>, mut next: ResMut<NextState<GameState>>) {
    if matches!(state.get(), GameState::Loading) {
        // Immediately transition to Playing (placeholder for future asset gating)
        next.set(GameState::Playing);
    }
}

// -- Web-only helpers to make the game fill the browser and support fullscreen --
#[cfg(target_arch = "wasm32")]
fn resize_canvas_to_window(
    mut windows: Query<&mut bevy::window::Window, With<PrimaryWindow>>,
) {
    // Match the logical resolution to the current window size; CSS is already set via index.html to 100vw/100vh
    if let Ok(mut win) = windows.get_single_mut() {
        if let Some(w) = web_window() {
            if let Some(doc) = w.document() {
                if let Some(elem) = doc.get_element_by_id("bevy") {
                    if let Ok(canvas) = elem.dyn_into::<web_sys::HtmlCanvasElement>() {
                        // Set CSS size to fill the viewport (redundant with index.html but safe)
                        canvas.style().set_property("width", "100vw").ok();
                        canvas.style().set_property("height", "100vh").ok();
                        // Ensure the canvas is anchored to the viewport origin to avoid offset hitboxes
                        canvas.style().set_property("position", "fixed").ok();
                        canvas.style().set_property("top", "0").ok();
                        canvas.style().set_property("left", "0").ok();
                        canvas.style().set_property("touch-action", "none").ok();
                    }
                }
            }
            let width = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1280.0) as f32;
            let height = w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(720.0) as f32;
            // Only update if changed to avoid churn
            if win.resolution.width() != width || win.resolution.height() != height {
                win.resolution.set(width, height);
            }
            // Force a 1.0 scale factor so Bevy logical coordinates match CSS pixels on web
            if win.scale_factor_override != Some(1.0) {
                win.scale_factor_override = Some(1.0);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn toggle_fullscreen(
    kb: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut bevy::window::Window, With<PrimaryWindow>>,
) {
    if kb.just_pressed(KeyCode::F11) || kb.just_pressed(KeyCode::KeyF) {
        if let Ok(mut win) = windows.get_single_mut() {
            win.mode = if matches!(win.mode, WindowMode::Fullscreen) {
                WindowMode::Windowed
            } else {
                WindowMode::Fullscreen
            };
        }
    }
}
