use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::states::GameState;
use super::player::TouchState;

pub struct DebugPlugin;

#[derive(Resource, Default)]
struct DebugOverlay(bool);

#[derive(Component)]
struct DebugHudText;

#[derive(Component)]
struct PointerMarker;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugOverlay>()
            .add_systems(Update, toggle_debug_overlay)
            .add_systems(
                OnEnter(GameState::Playing),
                (spawn_debug_hud, spawn_pointer_marker),
            )
            .add_systems(
                Update,
                (
                    update_debug_hud,
                    update_pointer_marker,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn toggle_debug_overlay(kb: Res<ButtonInput<KeyCode>>, mut overlay: ResMut<DebugOverlay>) {
    if kb.just_pressed(KeyCode::F3) {
        overlay.0 = !overlay.0;
    }
}

fn spawn_debug_hud(mut commands: Commands) {
    commands.spawn((
        DebugHudText,
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(8.0),
                bottom: Val::Px(8.0),
                ..default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..default()
        },
    ))
    .with_children(|p| {
        p.spawn(TextBundle::from_section(
            "",
            TextStyle {
                font: default(),
                font_size: 12.0,
                color: Color::rgb(0.9, 0.9, 0.9),
            },
        ));
    });
}

fn update_debug_hud(
    overlay: Res<DebugOverlay>,
    mut q_text: Query<&mut Text, With<DebugHudText>>, 
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if overlay.is_changed() || overlay.0 {
        if let Ok(win) = windows.get_single() {
            if let Ok(mut text) = q_text.get_single_mut() {
                if overlay.0 {
                    let logical = Vec2::new(win.resolution.width(), win.resolution.height());
                    let scale = win.resolution.scale_factor() as f32;
                    let physical = logical * scale;
                    let cursor_l = win.cursor_position().map(|v| format!("{:.1},{:.1}", v.x, v.y)).unwrap_or_else(|| "-".into());
                    let cursor_p = win
                        .physical_cursor_position()
                        .map(|v| format!("{:.1},{:.1}", v.x, v.y))
                        .unwrap_or_else(|| "-".into());
                    text.sections[0].value = format!(
                        "win logical: {:.1}x{:.1}\nwin physical: {:.1}x{:.1} (scale {:.2})\ncursor L/P: {} | {}",
                        logical.x, logical.y, physical.x, physical.y, scale, cursor_l, cursor_p
                    );
                } else {
                    text.sections[0].value.clear();
                }
            }
        }
    }
}

fn spawn_pointer_marker(mut commands: Commands) {
    commands.spawn((
        PointerMarker,
        SpriteBundle {
            sprite: Sprite {
                color: Color::YELLOW,
                custom_size: Some(Vec2::splat(6.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 999.0),
            ..default()
        },
    ));
}

fn update_pointer_marker(
    overlay: Res<DebugOverlay>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    touch: Res<TouchState>,
    mut marker_q: Query<(&mut Transform, &mut Visibility), With<PointerMarker>>,
) {
    if !overlay.0 { return; }
    if let (Ok(win), Ok((camera, cam_tf)), Ok((mut tf, mut vis))) = (windows.get_single(), camera_q.get_single(), marker_q.get_single_mut()) {
        let pointer_phys = if touch.is_active() {
            touch.position().map(|mut p| { p *= win.resolution.scale_factor() as f32; p })
        } else {
            win.physical_cursor_position().map(|p| Vec2::new(p.x as f32, p.y as f32))
        };
        if let Some(pos) = pointer_phys {
            if let Some(world) = camera.viewport_to_world_2d(cam_tf, pos) {
                tf.translation.x = world.x;
                tf.translation.y = world.y;
                *vis = Visibility::Visible;
                return;
            }
        }
        *vis = Visibility::Hidden;
    }
}
