# Pixel Alchemy

[![CI & Deploy Pages](https://github.com/Dafenxz0/pixel-alchemy/actions/workflows/ci.yml/badge.svg)](https://github.com/Dafenxz0/pixel-alchemy/actions/workflows/ci.yml)

**Live Demo:** https://dafenxz0.github.io/pixel-alchemy/

Noita-style cellular pixel physics, multi-fluid density displacement, 2D thermodynamics, electrical BFS conduction, and chemical reaction engine written in Rust and compiled directly to `wasm32-unknown-unknown`.

## Architecture

- **Rust Simulation Core (`src/`)**
  - [`src/material.rs`](src/material.rs): 22 materials across 4 active physical phases (Powder, Liquid, Gas, Solid/Energy) with density, horizontal dispersion, thermal conductivity, flammability, acid susceptibility, and electrical conductivity tables.
  - [`src/sim.rs`](src/sim.rs): `320×200` double-buffered 2D thermal diffusion (`-50°C` to `1500°C`) with heat advection on particle movement, multi-fluid density swapping, upper-atmosphere steam condensation, BFS electrical propagation along `Metal`/`Saltwater`/`Water`, iterative blast shockwaves, and `RGBA` + `Thermal IR` rendering.
  - [`src/presets.rs`](src/presets.rs): 6 interactive scene presets (*Volcanic Hydro-Cycle*, *Chemical Reactor*, *Circuit Bomb*, *Rainforest Fire*, *Oil & Acid Tank*, *Bio-Hazard Lab*).
  - [`src/lib.rs`](src/lib.rs): Zero-dependency `#[no_mangle] pub extern "C"` FFI exports (`wasm_init`, `wasm_step`, `wasm_paint_line`, `wasm_thermal_brush`, `wasm_rgba_ptr`, `wasm_cell_material`, `wasm_cell_temp`, telemetry getters) and `cargo test` physics suite.
- **Lab Web UI (`web/`)**
  - Full-bleed crisp `image-rendering: pixelated` canvas (`web/index.html`, `web/styles.css`, `web/app.js`).
  - Live hover cell inspector (`x`, `y`, material, phase, temperature `°C`), Heat (`+120°C`) & Cryo (`-80°C`) thermal brushes, Material vs. Thermal IR view toggle, variable speed (`0.5×`–`4×`), single-step debugging, and phase distribution telemetry.

## Element Reaction Matrix

| Material | Phase | Density | Key Interactions & Phase Transitions |
| --- | --- | ---: | --- |
| **Sand** | Powder | 180 | Falls and piles; sinks cleanly through Oil, Water, Saltwater, Acid, Nitro |
| **Gunpowder** | Powder | 150 | Sinks in liquids; detonates `>= 220°C` or on Fire / Spark / Lava |
| **Virus** | Powder | 130 | Infects adjacent `Plant` & `Wood`; killed by `Fire`, `Lava`, `Acid`, or `>= 120°C` |
| **Oil** | Liquid | 75 | Floats above Water; ignites `>= 240°C` into sustained Fire & Smoke |
| **Water** | Liquid | 100 | Boils `>= 100°C` to Steam, freezes `<= 0°C` to Ice, grows Plant, conducts Spark |
| **Saltwater** | Liquid | 114 | Sinks below Water; boils `>= 106°C`, freezes `<= -12°C`, conducts Spark strongly |
| **Acid** | Liquid | 124 | Dissolves Wood, Metal, Stone, Plant, Virus & Ice into `ToxicGas` (consumed stochastically) |
| **Nitro** | Liquid | 138 | High-explosive liquid; radial shockwave + shrapnel `>= 110°C` or on Fire/Spark/Acid/Lava |
| **Lava** | Liquid | 220 | `1150°C` magma; boils Water, melts Ice, ignites fuels, quenches to `Obsidian` (with Water/Ice) or `Stone` |
| **Steam** | Gas | -20 | Rises and cools aloft; condenses `<= 82°C` back into falling `Water` rain |
| **Smoke** | Gas | -14 | Rises, disperses laterally with turbulence, and fades |
| **Methane** | Gas | -28 | Buoyant flammable gas; flashes into fireball `>= 200°C` or on Fire/Spark |
| **ToxicGas** | Gas | -10 | Corrosive green vapor emitted by Acid reactions |
| **Ice** | Solid | 900 | `-20°C` chiller; condenses Steam, quenches Lava to Obsidian, melts `>= 2°C` |
| **Wood** | Solid | 1000 | Flammable timber (`>= 260°C`); dissolved by Acid, infected by Virus |
| **Plant** | Solid | 1000 | Living flora; consumes touching `Water` to grow, burns `>= 180°C` |
| **Stone** | Solid | 1000 | Refractory rock; slowly dissolved by Acid, shattered by Nitro core blasts |
| **Obsidian** | Solid | 1000 | Volcanic glass; acid-proof and blast-resistant |
| **Metal** | Solid | 1000 | High thermal conductivity; propagates `Spark` via BFS across entire circuits |
| **Fire** | Energy | -30 | `650°C` flickering flame; ignites flammables, extinguished by Water/Ice |
| **Spark** | Energy | 0 | Electrical plasma arc; energizes `Metal`, `Saltwater`, and `Water` conductors |

## Controls

- **Left-drag**: Paint selected element or apply Heat/Cryo thermal brush
- **Right-drag / Shift+drag**: Erase to `Empty`
- **Hover**: Inspect exact cell material, phase, and temperature (`°C`)
- **Shortcuts**: `Space` Play/Pause, `.` Step +1 tick, `T` Material/Thermal IR view, `R` Reset preset, `C` Clear, `[` / `]` Brush size

## Building Locally

```bash
cargo test
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/pixel_alchemy.wasm web/pixel_alchemy.wasm
python3 -m http.server -d web 8080
```

## License

[MIT](LICENSE)
