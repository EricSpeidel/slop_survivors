# Slop Survivors (Bevy 0.13)

Dragon-themed survivors-like prototype built with Bevy 0.13.

## Prerequisites
- Rust (stable toolchain; install via https://rustup.rs)
- On Windows ensure latest GPU drivers (Vulkan backend used by default)

Optional (for faster incremental compiles):
- `cargo install cargo-watch` for live-reload style runs.

## Build & Run
```
cargo build
cargo run
```
A window titled "Slop Survivors" should appear (1280x720). Use WASD to move the player.

## Controls
- WASD: Move

## Current Gameplay Loop
- Player spawns with a blue square and a camera
- Enemies spawn at screen edges and seek the player
- When enemies reach the player radius they are auto-destroyed and leave XP orbs
- XP orbs picked up increase XP (HUD updates)

## Code Structure
- `src/main.rs` sets up window + `GamePlugin`
- `game/states.rs` defines `GameState`
- `game/player.rs` player spawn & movement
- `game/enemy.rs` enemy seeking logic
- `game/spawn.rs` timed enemy spawns
- `game/combat.rs` simple collision-based kill + XP spawn
- `game/xp.rs` XP orb entity & pickup system
- `game/ui.rs` HUD for XP / HP
- `game/assets.rs` placeholder for future asset loading

## Updating Bevy
This project targets Bevy 0.13. To attempt an upgrade later:
1. Change `bevy` version in `Cargo.toml`
2. Adjust APIs (notably: Time delta methods, Color constructors, State API, Text API) per Bevy release notes.

## Potential Next Improvements
- Proper damage & health interactions instead of instant enemy removal
- Weapon system (projectiles, cooldowns)
- Wave scaling / difficulty curve
- Pause & game over states (implement `Paused`, `GameOver` transitions)
- Asset loading (spritesheets, fonts) in `assets.rs`
- Audio feedback (spawn/collect/level-up)
- Upgrade system on level thresholds
- Performance: switch to release profile with `cargo run --release`

## Troubleshooting
- If window does not open: update graphics drivers, try forcing backend: `RUST_LOG=info WGPU_BACKEND=dx12 cargo run`
- If build is slow: enable incremental, or run `cargo check` while iterating.

## License
MIT OR Apache-2.0 (dual).
