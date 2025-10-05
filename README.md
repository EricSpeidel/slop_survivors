# Slop Survivors (Working Title)

A dragon-themed "survivors-like" inspired by Vampire Survivors, built in Rust with the Bevy engine.

## Vision
You are a wyrmling growing in power while fending off relentless waves of fantasy hunters and rival drakes. Collect treasure embers (XP), evolve breath weapons, and unlock ancient dragon aspects.

## Current Status
Early scaffold. Basic player/enemy prototypes incoming.

## Tech Stack
- Rust
- Bevy 0.14 (ECS + Renderer + Audio + UI)
- `bevy-inspector-egui` (debug utilities)

## Running
```powershell
# Build & run (dynamic linking speeds up dev on Windows)
cargo run

# Faster iterative compile (optional features trimming example)
# cargo run --no-default-features --features minimal
```

## Run in the browser (WebAssembly)

Prerequisites (first time):

- Install Rust WASM target and Trunk (static file server & bundler):

```
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Then run in dev mode (serves at http://127.0.0.1:8080 by default):

```
trunk serve --release
```

Or build the web bundle to `dist/`:

```
trunk build --release
```

Notes:

- We include an `index.html` with `<link data-trunk rel="copy-dir" href="assets" />` so assets are available under the same root.
- Bevy 0.13 defaults work on WebGL2. Avoid features not supported on web (e.g., some audio backends) or gate them behind `#[cfg(not(target_arch = "wasm32"))]` if needed.

## Roadmap (High-Level)
1. Core movement & camera
2. Enemy spawning & pathing
3. XP / leveling loop
4. Procedural weapon evolutions (breath cones, tail sweeps, wing buffets)
5. Rarity & relic system
6. Biome progression and meta unlocks

## Repo Structure (planned)
```
src/
  main.rs
  game/
    mod.rs
    states.rs
    player.rs
    enemy.rs
    spawn.rs
    movement.rs
    xp.rs
    ui.rs
    assets.rs
    combat.rs
```

## License
Dual-licensed under MIT or Apache-2.0.

---
*Replace placeholder author info in Cargo.toml before publishing.*
