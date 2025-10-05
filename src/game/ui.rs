use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::window::PrimaryWindow;

use super::player::PlayerStats;
use super::states::GameState;
use super::player::{Player, PlayerAura, OrbitingFlame};
use super::xp::PendingLevelUps;
use super::combat::{AuraConfig, AuraTickTimer};

pub struct UiPlugin;

#[derive(Component)]
struct HudRoot;

// Bar fill markers
#[derive(Component)]
struct HpBarFill;
#[derive(Component)]
struct XpBarFill;

// Text markers
#[derive(Component)]
struct HpText;
#[derive(Component)]
struct XpText;
#[derive(Component)]
struct LevelText;
#[derive(Component)]
struct FpsText;
#[derive(Component)]
struct LevelUpText;
#[derive(Component)]
struct LevelUpHeaderText;
#[derive(Component)]
struct LevelUpChoiceText;

// Tracks cumulative flame speed multiplier so newly spawned flames match existing upgrades
#[derive(Resource, Deref, DerefMut)]
struct FlameSpeedBuff(f32);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FlameSpeedBuff(1.0))
            .add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(OnEnter(GameState::LevelUp), levelup_spawn_overlay_now)
            .add_systems(OnExit(GameState::LevelUp), levelup_cleanup_overlay)
            .add_systems(Update, (
                update_hud_bars.run_if(in_state(GameState::Playing).or_else(in_state(GameState::GameOver))),
                update_hud_text.run_if(in_state(GameState::Playing).or_else(in_state(GameState::GameOver))),
                update_fps_text,
                show_game_over_overlay.run_if(in_state(GameState::GameOver)),
                levelup_show_overlay.run_if(in_state(GameState::LevelUp)),
                levelup_handle_buttons.run_if(in_state(GameState::LevelUp)),
                responsive_levelup_overlay.run_if(in_state(GameState::LevelUp)),
                levelup_button_visuals.run_if(in_state(GameState::LevelUp)),
            ));
    }
}

fn setup_hud(mut commands: Commands, existing: Query<Entity, With<HudRoot>>) {
    if existing.get_single().is_ok() { return; }
    // Root full-screen UI node
    commands.spawn((HudRoot, NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(8.0)),
            row_gap: Val::Px(6.0),
            ..default()
        },
        ..default()
    })).with_children(|parent| {
        // HP Bar container (background)
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(220.0),
                height: Val::Px(18.0),
                ..default()
            },
            background_color: BackgroundColor(Color::rgb(0.15, 0.05, 0.05)),
            ..default()
        }).with_children(|hp_container| {
            hp_container.spawn((HpBarFill, NodeBundle {
                style: Style {
                    width: Val::Px(220.0), // full initially
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgb(0.9, 0.1, 0.1)),
                ..default()
            }));
        });

        // XP Bar container
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(220.0),
                height: Val::Px(12.0),
                ..default()
            },
            background_color: BackgroundColor(Color::rgb(0.12, 0.10, 0.18)),
            ..default()
        }).with_children(|xp_container| {
            xp_container.spawn((XpBarFill, NodeBundle {
                style: Style {
                    width: Val::Px(0.0), // starts empty until first XP
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgb(0.8, 0.7, 0.2)),
                ..default()
            }));
        });

        // Text rows (use Bevy's built-in default font)
        parent.spawn((HpText, TextBundle::from_section(
            "HP: 0 / 0",
            TextStyle { font: default(), font_size: 16.0, color: Color::WHITE }
        )));

        parent.spawn((LevelText, TextBundle::from_section(
            "Level: 1",
            TextStyle { font: default(), font_size: 16.0, color: Color::WHITE }
        )));

        parent.spawn((XpText, TextBundle::from_section(
            "XP: 0",
            TextStyle { font: default(), font_size: 16.0, color: Color::WHITE }
        )));

        // FPS in top-right corner
        parent.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(8.0),
                top: Val::Px(8.0),
                ..default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..default()
        }).with_children(|fps_node| {
            fps_node.spawn((FpsText, TextBundle::from_section(
                "FPS: --",
                TextStyle { font: default(), font_size: 14.0, color: Color::WHITE }
            )));
        });
    });
}

fn update_hud_bars(stats: Res<PlayerStats>, mut hp_fill: Query<&mut Style, With<HpBarFill>>, mut xp_fill: Query<&mut Style, (With<XpBarFill>, Without<HpBarFill>)>) {
    if !stats.is_changed() { return; }
    let hp_ratio = if stats.max_hp > 0.0 { (stats.hp / stats.max_hp).clamp(0.0, 1.0) } else { 0.0 };
    if let Ok(mut style) = hp_fill.get_single_mut() {
        style.width = Val::Px(220.0 * hp_ratio as f32);
    }
    // XP progress toward next level (each 100 xp per current formula)
    let xp_into_level = stats.xp % 100; // 0..99
    let xp_ratio = (xp_into_level as f32 / 100.0).clamp(0.0, 1.0);
    if let Ok(mut style) = xp_fill.get_single_mut() {
        style.width = Val::Px(220.0 * xp_ratio);
    }
}

fn levelup_button_visuals(
    mut q: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&BtnMoreDamage>,
            Option<&BtnMoreFlame>,
            Option<&BtnFasterFlames>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg, is_damage, is_flame, is_speed) in q.iter_mut() {
        let (base, hover, pressed) = if is_damage.is_some() {
            (
                Color::rgb(0.32, 0.12, 0.12),
                Color::rgb(0.42, 0.17, 0.17),
                Color::rgb(0.52, 0.22, 0.22),
            )
        } else if is_flame.is_some() {
            (
                Color::rgb(0.12, 0.32, 0.12),
                Color::rgb(0.17, 0.42, 0.17),
                Color::rgb(0.22, 0.52, 0.22),
            )
        } else if is_speed.is_some() {
            (
                Color::rgb(0.12, 0.12, 0.32),
                Color::rgb(0.17, 0.17, 0.42),
                Color::rgb(0.22, 0.22, 0.52),
            )
        } else {
            // Fallback palette (shouldn't happen for our buttons)
            (
                Color::rgb(0.2, 0.2, 0.2),
                Color::rgb(0.3, 0.3, 0.3),
                Color::rgb(0.4, 0.4, 0.4),
            )
        };

        *bg = match *interaction {
            Interaction::Pressed => BackgroundColor(pressed),
            Interaction::Hovered => BackgroundColor(hover),
            Interaction::None => BackgroundColor(base),
        };
    }
}

fn update_hud_text(
    stats: Res<PlayerStats>,
    mut hp_text_q: Query<&mut Text, With<HpText>>,
    mut xp_text_q: Query<&mut Text, (With<XpText>, Without<HpText>)>,
    mut lvl_text_q: Query<&mut Text, (With<LevelText>, Without<HpText>, Without<XpText>)>,
) {
    if !stats.is_changed() { return; }
    if let Ok(mut text) = hp_text_q.get_single_mut() {
        text.sections[0].value = format!("HP: {} / {}", stats.hp as i32, stats.max_hp as i32);
    }
    if let Ok(mut text) = lvl_text_q.get_single_mut() {
        text.sections[0].value = format!("Level: {}", stats.level);
    }
    if let Ok(mut text) = xp_text_q.get_single_mut() {
        text.sections[0].value = format!("XP: {}", stats.xp);
    }
}

#[derive(Component)]
struct GameOverOverlay;

#[derive(Component)]
struct LevelUpOverlay;
#[derive(Component)]
struct BtnMoreDamage;
#[derive(Component)]
struct BtnMoreFlame;
#[derive(Component)]
struct BtnFasterFlames;

fn show_game_over_overlay(mut commands: Commands, root: Query<Entity, With<HudRoot>>, existing: Query<Entity, With<GameOverOverlay>>) {
    if existing.get_single().is_ok() { return; }
    let Ok(root_entity) = root.get_single() else { return; };
    commands.entity(root_entity).with_children(|parent| {
        // Semi-transparent panel to indicate game over (no font usage)
        parent.spawn((GameOverOverlay, NodeBundle {
            style: Style {
                width: Val::Px(260.0),
                height: Val::Px(60.0),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.2, 0.0, 0.0, 0.75)),
            ..default()
        }));
    });
}

fn levelup_show_overlay(
    commands: Commands,
    root: Query<Entity, With<HudRoot>>,
    existing: Query<Entity, With<LevelUpOverlay>>,
) {
    if existing.get_single().is_ok() { return; }
    let Ok(root_entity) = root.get_single() else { return; };
    spawn_levelup_overlay(commands, root_entity);
}

fn levelup_spawn_overlay_now(
    mut commands: Commands,
    root: Query<Entity, With<HudRoot>>,
    existing: Query<Entity, With<LevelUpOverlay>>,
) {
    // Clean any stale overlay then spawn fresh
    for e in existing.iter() { commands.entity(e).despawn_recursive(); }
    if let Ok(root_entity) = root.get_single() {
        spawn_levelup_overlay(commands, root_entity);
    }
}

fn levelup_cleanup_overlay(mut commands: Commands, existing: Query<Entity, With<LevelUpOverlay>>) {
    for e in existing.iter() { commands.entity(e).despawn_recursive(); }
}

fn spawn_levelup_overlay(mut commands: Commands, root_entity: Entity) {
    commands.entity(root_entity).with_children(|parent| {
        parent.spawn((LevelUpOverlay, NodeBundle {
            style: Style {
                width: Val::Px(520.0),
                height: Val::Auto, // let content determine height so buttons are fully inside for touch hit-testing
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                row_gap: Val::Px(12.0),
                margin: UiRect::all(Val::Auto),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.86)),
            z_index: ZIndex::Global(100),
            ..default()
        })).with_children(|p| {
            p.spawn((LevelUpText, LevelUpHeaderText, TextBundle::from_section(
                "Level Up! Choose an upgrade:",
                TextStyle { font: default(), font_size: 22.0, color: Color::YELLOW }
            )));
            // Buttons (full-width by default; responsive system may adjust)
            let button_style = Style { width: Val::Percent(100.0), height: Val::Px(44.0), ..default() };
            p.spawn((BtnMoreDamage, ButtonBundle {
                style: button_style.clone(),
                background_color: BackgroundColor(Color::rgb(0.32, 0.12, 0.12)),
                ..default()
            })).with_children(|b| {
                b.spawn((LevelUpText, LevelUpChoiceText, TextBundle::from_section(
                    "+5 Aura Damage",
                    TextStyle { font: default(), font_size: 18.0, color: Color::WHITE }
                )));
            });
            p.spawn((BtnMoreFlame, ButtonBundle {
                style: button_style.clone(),
                background_color: BackgroundColor(Color::rgb(0.12, 0.32, 0.12)),
                ..default()
            })).with_children(|b| {
                b.spawn((LevelUpText, LevelUpChoiceText, TextBundle::from_section(
                    "+1 Flame",
                    TextStyle { font: default(), font_size: 18.0, color: Color::WHITE }
                )));
            });
            p.spawn((BtnFasterFlames, ButtonBundle {
                style: button_style,
                background_color: BackgroundColor(Color::rgb(0.12, 0.12, 0.32)),
                ..default()
            })).with_children(|b| {
                b.spawn((LevelUpText, LevelUpChoiceText, TextBundle::from_section(
                    "+20% Flame Speed (and tick)",
                    TextStyle { font: default(), font_size: 18.0, color: Color::WHITE }
                )));
            });
        });
    });
}

fn levelup_handle_buttons(
    mut commands: Commands,
    mut pending: ResMut<PendingLevelUps>,
    mut next: ResMut<NextState<GameState>>,
    mut cfg: ResMut<AuraConfig>,
    player_q: Query<Entity, With<Player>>, // parent to attach new flames
    mut flames_q: Query<&mut OrbitingFlame, With<OrbitingFlame>>,
    flames_count_q: Query<(), With<OrbitingFlame>>,
    mut ui_entities: Query<Entity, With<LevelUpOverlay>>,
    // Separate queries to detect which specific button was pressed
    q_more_damage: Query<&Interaction, (Changed<Interaction>, With<Button>, With<BtnMoreDamage>)>,
    q_more_flame: Query<&Interaction, (Changed<Interaction>, With<Button>, With<BtnMoreFlame>)>,
    q_faster: Query<&Interaction, (Changed<Interaction>, With<Button>, With<BtnFasterFlames>)>,
    player_assets: Res<crate::game::assets::PlayerAssets>,
    mut aura_timer: ResMut<AuraTickTimer>,
    root_q: Query<Entity, With<HudRoot>>,
    mut flame_speed: ResMut<FlameSpeedBuff>,
) {
    // Option 1: +5 aura damage
    for interaction in q_more_damage.iter() {
        if *interaction == Interaction::Pressed {
            cfg.damage_tick += 5.0;
            // finalize selection
            if pending.0 > 0 { pending.0 -= 1; }
            for e in ui_entities.iter_mut() { commands.entity(e).despawn_recursive(); }
            if pending.0 > 0 {
                if let Ok(root) = root_q.get_single() { spawn_levelup_overlay(commands.reborrow(), root); }
                next.set(GameState::LevelUp);
            } else { next.set(GameState::Playing); }
            return;
        }
    }

    // Option 2: +1 flame and redistribute
    for interaction in q_more_flame.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(player_entity) = player_q.get_single() {
                commands.entity(player_entity).with_children(|p| {
                    let angle = 0.0;
                    // Ensure new flame speed matches existing upgrades via FlameSpeedBuff
                    let base_speed = 1.8 * **flame_speed;
                    p.spawn((PlayerAura, OrbitingFlame { angle, radius: cfg.radius, speed: base_speed, contact_radius: 16.0, contact_damage: 5.0 }, SpriteBundle {
                        texture: player_assets.flame.clone(),
                        sprite: Sprite { color: Color::rgba(1.0, 1.0, 1.0, 0.0), custom_size: Some(Vec2::splat(32.0)), ..default() },
                        transform: Transform::from_xyz(angle.cos() * cfg.radius, angle.sin() * cfg.radius, 0.0),
                        ..default()
                    }));
                });
                // Redistribute angles evenly among all flames
                let n = flames_count_q.iter().count().max(1) as f32;
                let mut idx = 0usize;
                for mut of in flames_q.iter_mut() {
                    of.angle = idx as f32 / n * std::f32::consts::TAU;
                    of.radius = cfg.radius;
                    idx += 1;
                }
            }
            if pending.0 > 0 { pending.0 -= 1; }
            for e in ui_entities.iter_mut() { commands.entity(e).despawn_recursive(); }
            if pending.0 > 0 {
                if let Ok(root) = root_q.get_single() { spawn_levelup_overlay(commands.reborrow(), root); }
                next.set(GameState::LevelUp);
            } else { next.set(GameState::Playing); }
            return;
        }
    }

    // Option 3: +20% flame speed
    for interaction in q_faster.iter() {
        if *interaction == Interaction::Pressed {
            for mut of in flames_q.iter_mut() {
                of.speed *= 1.20;
            }
            // Update global buff so future flames inherit the current speed multiplier
            **flame_speed *= 1.20;
            // Also decrease aura damage tick interval by 20% (faster ticks), with a reasonable floor
            let current = aura_timer.duration_secs();
            let new = (current * 0.8).max(0.05);
            aura_timer.set_duration_secs(new);
            if pending.0 > 0 { pending.0 -= 1; }
            for e in ui_entities.iter_mut() { commands.entity(e).despawn_recursive(); }
            if pending.0 > 0 {
                if let Ok(root) = root_q.get_single() { spawn_levelup_overlay(commands.reborrow(), root); }
                next.set(GameState::LevelUp);
            } else { next.set(GameState::Playing); }
            return;
        }
    }
}

fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut q: Query<&mut Text, With<FpsText>>,
) {
    if let Ok(mut text) = q.get_single_mut() {
        if let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(avg) = fps_diag.smoothed() {
                text.sections[0].value = format!("FPS: {:.0}", avg.max(0.0));
            }
        }
    }
}

fn responsive_levelup_overlay(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut overlay_q: Query<&mut Style, With<LevelUpOverlay>>,
    mut button_q: Query<&mut Style, (With<Button>, Or<(With<BtnMoreDamage>, With<BtnMoreFlame>, With<BtnFasterFlames>)>, Without<LevelUpOverlay>)>,
    mut header_q: Query<&mut Text, (With<LevelUpHeaderText>, Without<LevelUpChoiceText>)>,
    mut choice_q: Query<&mut Text, (With<LevelUpChoiceText>, Without<LevelUpHeaderText>)>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let width = window.resolution.width();
    let (overlay_w, button_h, header_fs, choice_fs, button_w): (Val, f32, f32, f32, Val) = if width < 520.0 {
        (Val::Percent(95.0), 36.0, 18.0, 14.0, Val::Percent(100.0))
    } else if width < 820.0 {
        (Val::Percent(80.0), 40.0, 20.0, 16.0, Val::Percent(100.0))
    } else {
        (Val::Px(520.0), 44.0, 22.0, 18.0, Val::Px(480.0))
    };
    if let Ok(mut style) = overlay_q.get_single_mut() { style.width = overlay_w; }
    for mut s in button_q.iter_mut() { s.width = button_w; s.height = Val::Px(button_h); }
    if let Ok(mut t) = header_q.get_single_mut() { t.sections[0].style.font_size = header_fs; }
    for mut t in choice_q.iter_mut() { t.sections[0].style.font_size = choice_fs; }
}
