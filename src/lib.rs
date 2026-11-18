//! Rust -> WASM Cellular Pixel Physics, Multi-Fluid Dynamics, 2D Thermodynamics,
//! and Chemistry Engine.

pub mod material;
pub mod presets;
pub mod sim;

use std::sync::Mutex;

use material::Material;
use sim::{World, HEIGHT, WIDTH};

static WORLD: Mutex<Option<World>> = Mutex::new(None);

fn with_world<R>(f: impl FnOnce(&mut World) -> R) -> Option<R> {
    let mut guard = WORLD.lock().ok()?;
    let world = guard.get_or_insert_with(|| World::new(0x5A17_C0DE));
    Some(f(world))
}

#[no_mangle]
pub extern "C" fn wasm_init(seed: u32) {
    if let Ok(mut guard) = WORLD.lock() {
        *guard = Some(World::new(seed));
    }
}

#[no_mangle]
pub extern "C" fn wasm_width() -> u32 {
    WIDTH as u32
}

#[no_mangle]
pub extern "C" fn wasm_height() -> u32 {
    HEIGHT as u32
}

#[no_mangle]
pub extern "C" fn wasm_step(steps: u32) {
    with_world(|w| w.step(steps.clamp(1, 16)));
}

#[no_mangle]
pub extern "C" fn wasm_clear() {
    with_world(World::clear);
}

#[no_mangle]
pub extern "C" fn wasm_load_preset(preset_id: u32) {
    with_world(|w| presets::load_preset(w, preset_id));
}

#[no_mangle]
pub extern "C" fn wasm_paint(x: i32, y: i32, radius: i32, material_id: u8) {
    with_world(|w| {
        w.paint(x, y, radius, Material::from_u8(material_id));
        w.render();
    });
}

#[no_mangle]
pub extern "C" fn wasm_paint_line(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    radius: i32,
    material_id: u8,
) {
    with_world(|w| {
        w.paint_line(x0, y0, x1, y1, radius, Material::from_u8(material_id));
        w.render();
    });
}

#[no_mangle]
pub extern "C" fn wasm_thermal_brush(x: i32, y: i32, radius: i32, delta_temp: f32) {
    with_world(|w| {
        w.thermal_brush(x, y, radius, delta_temp);
        w.render();
    });
}

#[no_mangle]
pub extern "C" fn wasm_set_view_mode(mode: u32) {
    with_world(|w| {
        w.view_mode = mode;
        w.render();
    });
}

#[no_mangle]
pub extern "C" fn wasm_rgba_ptr() -> *const u8 {
    with_world(|w| w.rgba.as_ptr()).unwrap_or(std::ptr::null())
}

#[no_mangle]
pub extern "C" fn wasm_rgba_len() -> u32 {
    with_world(|w| w.rgba.len() as u32).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn wasm_materials_ptr() -> *const u8 {
    with_world(|w| w.materials.as_ptr()).unwrap_or(std::ptr::null())
}

#[no_mangle]
pub extern "C" fn wasm_temps_ptr() -> *const f32 {
    with_world(|w| w.temps.as_ptr()).unwrap_or(std::ptr::null())
}

#[no_mangle]
pub extern "C" fn wasm_cell_material(x: i32, y: i32) -> u8 {
    with_world(|w| w.material_at(x, y) as u8).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn wasm_cell_temp(x: i32, y: i32) -> f32 {
    with_world(|w| w.temp_at(x, y)).unwrap_or(sim::AMBIENT_TEMP)
}

#[no_mangle]
pub extern "C" fn wasm_tick_count() -> u32 {
    with_world(|w| w.tick_count).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn wasm_active_count() -> u32 {
    with_world(|w| w.active_count).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn wasm_reaction_count() -> u32 {
    with_world(|w| w.reaction_count).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn wasm_avg_temp() -> f32 {
    with_world(|w| w.avg_temp).unwrap_or(sim::AMBIENT_TEMP)
}

#[no_mangle]
pub extern "C" fn wasm_max_temp() -> f32 {
    with_world(|w| w.max_temp).unwrap_or(sim::AMBIENT_TEMP)
}

#[no_mangle]
pub extern "C" fn wasm_phase_count(phase: u32) -> u32 {
    with_world(|w| match phase {
        1..=4 => w.phase_counts[(phase - 1) as usize],
        _ => 0,
    })
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::{CELL_COUNT, HEIGHT, WIDTH};

    #[test]
    fn test_gravity_sand_falls_and_piles() {
        let mut w = World::new(42);
        w.set_cell(100, 10, Material::Sand);
        w.step(200);
        assert_eq!(w.material_at(100, 10), Material::Empty);
        assert_eq!(w.material_at(100, HEIGHT as i32 - 1), Material::Sand);
    }

    #[test]
    fn test_density_swapping_oil_over_water_and_sand_sinks() {
        let mut w = World::new(99);
        // Stone cup at bottom
        for x in 90..=110 {
            w.set_cell(x, 180, Material::Stone);
        }
        for y in 140..180 {
            w.set_cell(90, y, Material::Stone);
            w.set_cell(110, y, Material::Stone);
        }
        // Place Water ABOVE Oil inside the cup, plus Sand on top
        for y in 160..170 {
            for x in 92..=108 {
                w.set_cell(x, y, Material::Water);
            }
        }
        for y in 170..180 {
            for x in 92..=108 {
                w.set_cell(x, y, Material::Oil);
            }
        }
        w.set_cell(100, 155, Material::Sand);

        w.step(140);

        // Compute average Y of Oil vs Water vs Sand
        let mut oil_y_sum = 0usize;
        let mut oil_count = 0usize;
        let mut water_y_sum = 0usize;
        let mut water_count = 0usize;
        let mut sand_y = 0usize;
        for y in 140..180 {
            for x in 91..110 {
                match w.material_at(x, y as i32) {
                    Material::Oil => {
                        oil_y_sum += y;
                        oil_count += 1;
                    }
                    Material::Water => {
                        water_y_sum += y;
                        water_count += 1;
                    }
                    Material::Sand => {
                        sand_y = y;
                    }
                    _ => {}
                }
            }
        }

        assert!(oil_count > 50 && water_count > 50);
        let avg_oil_y = oil_y_sum as f32 / oil_count as f32;
        let avg_water_y = water_y_sum as f32 / water_count as f32;
        assert!(
            avg_oil_y < avg_water_y,
            "Oil (avg y={avg_oil_y}) should float above Water (avg y={avg_water_y})"
        );
        assert!(
            sand_y >= 176,
            "Sand (y={sand_y}) should sink to the bottom of the tank"
        );
    }

    #[test]
    fn test_boiling_and_condensation_cycle() {
        let mut w = World::new(7);
        // Build a closed Stone column so Steam rises to the cool top and condenses into Water
        for x in 148..=154 {
            w.set_cell(x, 180, Material::Stone);
            w.set_cell(x, 2, Material::Stone);
        }
        for y in 2..=180 {
            w.set_cell(148, y, Material::Stone);
            w.set_cell(154, y, Material::Stone);
        }
        // Lava heater directly under a Metal plate, Water above
        for x in 149..=153 {
            w.set_cell(x, 178, Material::Lava);
            w.set_cell(x, 177, Material::Lava);
            w.set_cell(x, 176, Material::Lava);
            w.set_cell(x, 175, Material::Metal);
            w.set_cell(x, 174, Material::Water);
            w.set_cell(x, 173, Material::Water);
        }

        let mut saw_steam_high = false;
        let mut saw_condensed_rain = false;
        for _ in 0..450 {
            w.step(1);
            for y in 4..48 {
                for x in 149..=153 {
                    let m = w.material_at(x, y);
                    if m == Material::Steam {
                        saw_steam_high = true;
                    }
                    if m == Material::Water && y < 48 {
                        saw_condensed_rain = true;
                    }
                }
            }
            if saw_steam_high && saw_condensed_rain {
                break;
            }
        }

        assert!(saw_steam_high, "Lava should boil Water into rising Steam");
        assert!(
            saw_condensed_rain,
            "Steam should cool at upper levels and condense back into falling Water rain"
        );
    }

    #[test]
    fn test_acid_corrosion_and_obsidian_resistance() {
        let mut w = World::new(123);
        // Obsidian basin holding Acid over a Wood/Metal block
        for x in 78..=86 {
            w.set_cell(x, 150, Material::Obsidian);
        }
        for y in 130..150 {
            w.set_cell(78, y, Material::Obsidian);
            w.set_cell(86, y, Material::Obsidian);
        }
        for x in 79..=85 {
            w.set_cell(x, 145, Material::Wood);
            w.set_cell(x, 144, Material::Metal);
            w.set_cell(x, 140, Material::Acid);
            w.set_cell(x, 141, Material::Acid);
        }

        w.step(90);

        let mut wood_or_metal_left = 0usize;
        let mut obsidian_intact = 0usize;
        let mut toxic_or_reactions = w.reaction_count;
        for x in 79..=85 {
            for y in 140..150 {
                let m = w.material_at(x, y);
                if m == Material::Wood || m == Material::Metal {
                    wood_or_metal_left += 1;
                }
            }
            if w.material_at(x, 150) == Material::Obsidian {
                obsidian_intact += 1;
            }
        }
        for &m in &w.materials {
            if m == Material::ToxicGas as u8 {
                toxic_or_reactions += 1;
            }
        }

        assert!(
            wood_or_metal_left < 14,
            "Acid should dissolve Wood and Metal"
        );
        assert_eq!(obsidian_intact, 7, "Obsidian must resist Acid");
        assert!(
            toxic_or_reactions > 0,
            "Acid corrosion should record reactions / emit ToxicGas"
        );
    }

    #[test]
    fn test_electrical_conduction_through_metal_ignites_gunpowder() {
        let mut w = World::new(55);
        // 40-cell horizontal Metal wire leading into Gunpowder at the far right
        for x in 60..=100 {
            w.set_cell(x, 100, Material::Metal);
        }
        w.set_cell(101, 100, Material::Gunpowder);
        w.set_cell(102, 100, Material::Gunpowder);
        // Spark placed on the far left of the Metal wire
        w.set_cell(60, 99, Material::Spark);

        w.step(3);

        // Far end of Metal wire should have been electrified and Gunpowder ignited
        let far_metal_life = w.life[World::idx(99, 100)];
        let gp_state = w.material_at(101, 100);
        assert!(
            far_metal_life > 0 || gp_state != Material::Gunpowder,
            "Electricity should propagate via BFS along Metal wire to ignite Gunpowder (life={far_metal_life}, gp={gp_state:?})"
        );
        assert_ne!(
            gp_state,
            Material::Gunpowder,
            "Gunpowder at far end of Metal wire should detonate"
        );
    }

    #[test]
    fn test_nitro_explosion_shockwave() {
        let mut w = World::new(321);
        for x in 145..=165 {
            w.set_cell(x, 120, Material::Wood);
        }
        for x in 153..=157 {
            for y in 115..=119 {
                w.set_cell(x, y, Material::Nitro);
            }
        }
        w.set_cell(155, 114, Material::Fire);
        w.step(8);

        let mut wood_intact = 0usize;
        for x in 150..=160 {
            if w.material_at(x, 120) == Material::Wood {
                wood_intact += 1;
            }
        }
        assert!(
            wood_intact < 11,
            "Nitro detonation should shatter nearby Wood floor (intact={wood_intact})"
        );
        assert!(
            w.max_temp > 400.0,
            "Nitro detonation should produce intense thermal spike (max_temp={})",
            w.max_temp
        );
    }

    #[test]
    fn test_virus_spread_and_plant_growth() {
        let mut w = World::new(77);
        for x in 48..=62 {
            w.set_cell(x, 101, Material::Stone);
        }
        for x in 50..=60 {
            w.set_cell(x, 100, Material::Plant);
        }
        w.set_cell(50, 100, Material::Virus);

        // Separate Plant + Water patch inside a Stone cup so Water stays contained
        for x in 199..=202 {
            w.set_cell(x, 100, Material::Stone);
        }
        w.set_cell(199, 99, Material::Stone);
        w.set_cell(202, 99, Material::Stone);
        w.set_cell(200, 99, Material::Plant);
        w.set_cell(201, 99, Material::Water);

        w.step(80);

        let mut virus_count = 0usize;
        for x in 50..=60 {
            if w.material_at(x, 100) == Material::Virus {
                virus_count += 1;
            }
        }
        assert!(virus_count >= 2, "Virus should infect adjacent Plant cells");
        assert_eq!(
            w.material_at(201, 99),
            Material::Plant,
            "Plant should grow into adjacent Water"
        );
    }

    #[test]
    fn test_determinism_and_presets_no_nans() {
        let mut a = World::new(2026);
        let mut b = World::new(2026);
        presets::load_preset(&mut a, 0);
        presets::load_preset(&mut b, 0);
        a.step(30);
        b.step(30);
        assert_eq!(a.materials, b.materials);
        assert_eq!(a.temps, b.temps);
        assert_eq!(a.rgba, b.rgba);

        for id in 0..presets::PRESET_COUNT {
            presets::load_preset(&mut a, id);
            assert!(a.active_count > 100, "Preset {id} should populate cells");
            a.step(10);
            assert!(!a.avg_temp.is_nan() && !a.max_temp.is_nan());
            for &t in &a.temps {
                assert!(t.is_finite(), "Preset {id} produced non-finite temp");
                assert!((sim::MIN_TEMP..=sim::MAX_TEMP).contains(&t));
            }
        }
        assert_eq!(WIDTH * HEIGHT, CELL_COUNT);
    }
}
