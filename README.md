# Bevy Doors

A small first-person horror hallway game inspired by [Roblox Doors](https://www.roblox.com/games/6516141723/DOORS), built with [Bevy](https://bevyengine.org/) for a class assignment.

## Gameplay

1. **Lobby** — Walk around or press **Space** to start a run.
2. **Run** — Move through endless hotel hallways. Each time you pass a numbered door, the door counter increases.
3. **Hazards** — Slow-moving entities chase you. Random jumpscare flashes can appear.
4. **Death** — Touching an entity ends the run. Press **Space** to return to the lobby.

## Controls

| Input | Action |
|-------|--------|
| WASD or Arrow keys | Move |
| Mouse | Look (during a run) |
| Space | Start run / return to lobby |
| Escape | Release mouse cursor |

## Requirements

- macOS (tested target)
- [Rust](https://rustup.rs/) (stable, 1.89+ for Bevy 0.18)

## Run

```bash
cargo run
```

Release builds run faster:

```bash
cargo run --release
```

## Project structure

- `src/main.rs` — App setup and game states
- `src/player.rs` — First-person movement and camera
- `src/lobby.rs` — Lobby room
- `src/hallway.rs` — Procedural hotel hallway segments
- `src/enemy.rs` — Chasing entities
- `src/jumpscare.rs` — Occasional screen flashes
- `src/ui.rs` — HUD and menus

## Notes

- Placeholder 3D meshes only (no custom assets).
- Silent prototype (no audio yet).
- Single-player, endless until death.
