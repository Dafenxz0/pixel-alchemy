//! Interactive scene presets showcasing thermodynamics, fluids, chemistry,
//! electrical BFS conduction, and biological spread.

use crate::material::Material;
use crate::sim::{World, HEIGHT, WIDTH};

pub const PRESET_COUNT: u32 = 6;

pub fn load_preset(world: &mut World, preset_id: u32) {
    world.clear();
    match preset_id % PRESET_COUNT {
        0 => preset_volcanic_hydro_cycle(world),
        1 => preset_chemical_reactor(world),
        2 => preset_circuit_bomb(world),
        3 => preset_rainforest_fire(world),
        4 => preset_oil_acid_tank(world),
        _ => preset_biohazard_lab(world),
    }
    world.render();
}

fn fill_rect(world: &mut World, x0: usize, y0: usize, x1: usize, y1: usize, mat: Material) {
    let x_end = x1.min(WIDTH);
    let y_end = y1.min(HEIGHT);
    for y in y0..y_end {
        for x in x0..x_end {
            world.set_cell(x, y, mat);
        }
    }
}

fn draw_thick_line(
    world: &mut World,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    radius: i32,
    mat: Material,
) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let cx = (x0 as f32 + (x1 - x0) as f32 * t).round() as i32;
        let cy = (y0 as f32 + (y1 - y0) as f32 * t).round() as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px >= 0 && py >= 0 && px < WIDTH as i32 && py < HEIGHT as i32 {
                        world.set_cell(px as usize, py as usize, mat);
                    }
                }
            }
        }
    }
}

fn fill_circle(world: &mut World, cx: i32, cy: i32, radius: i32, mat: Material) {
    let r2 = radius * radius;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= r2 {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && py >= 0 && px < WIDTH as i32 && py < HEIGHT as i32 {
                    world.set_cell(px as usize, py as usize, mat);
                }
            }
        }
    }
}

/// 0: Volcanic Hydro-Cycle — Lava boils Water into rising Steam, which cools at
/// the upper Ice/roof condenser and rains back down as falling Water.
fn preset_volcanic_hydro_cycle(world: &mut World) {
    // Outer stone chamber
    fill_rect(world, 0, 0, WIDTH, 6, Material::Stone);
    fill_rect(world, 0, HEIGHT - 10, WIDTH, HEIGHT, Material::Stone);
    fill_rect(world, 0, 0, 8, HEIGHT, Material::Stone);
    fill_rect(world, WIDTH - 8, 0, WIDTH, HEIGHT, Material::Stone);

    // Magma basin on lower-left and center
    fill_rect(world, 8, 152, 190, HEIGHT - 10, Material::Obsidian);
    fill_rect(world, 18, 158, 178, HEIGHT - 12, Material::Lava);

    // Metal heat-exchanger bridge over magma
    fill_rect(world, 22, 144, 174, 148, Material::Metal);

    // Water reservoir on top of the heat exchanger & right basin
    fill_rect(world, 26, 124, 170, 144, Material::Water);
    fill_rect(world, 190, 140, WIDTH - 8, HEIGHT - 10, Material::Water);

    // Ice condenser shelf near upper right to accelerate rain condensation
    draw_thick_line(world, 195, 34, 290, 44, 3, Material::Obsidian);
    fill_rect(world, 200, 22, 286, 31, Material::Ice);

    // Sloped stone catch-tray returning rain back toward the magma boiler
    draw_thick_line(world, 200, 108, 295, 92, 2, Material::Stone);
    draw_thick_line(world, 120, 72, 185, 84, 2, Material::Stone);
    fill_rect(world, 128, 66, 175, 71, Material::Ice);
}

/// 1: Chemical Reactor — Acid dissolves baffles into ToxicGas while Nitro/Methane
/// sit behind a sacrificial Wood plug.
fn preset_chemical_reactor(world: &mut World) {
    fill_rect(world, 0, HEIGHT - 8, WIDTH, HEIGHT, Material::Obsidian);
    fill_rect(world, 16, 20, 22, HEIGHT - 8, Material::Obsidian);
    fill_rect(
        world,
        WIDTH - 22,
        20,
        WIDTH - 16,
        HEIGHT - 8,
        Material::Obsidian,
    );

    // Upper left Acid vessel with Wood drip valve
    fill_rect(world, 22, 68, 118, 72, Material::Obsidian);
    fill_rect(world, 56, 68, 74, 72, Material::Wood);
    fill_rect(world, 24, 32, 116, 68, Material::Acid);

    // Reaction baffles (Stone, Metal, Wood)
    draw_thick_line(world, 24, 96, 124, 112, 2, Material::Stone);
    draw_thick_line(world, 48, 132, 152, 120, 2, Material::Metal);
    draw_thick_line(world, 24, 154, 142, 168, 2, Material::Wood);

    // Divider with Metal grate & Nitro/Methane side chamber
    fill_rect(world, 160, 48, 166, HEIGHT - 8, Material::Obsidian);
    fill_rect(world, 166, 140, WIDTH - 22, 144, Material::Obsidian);
    fill_rect(world, 174, 110, 232, 140, Material::Nitro);
    fill_rect(world, 236, 98, WIDTH - 26, 140, Material::Saltwater);
    fill_rect(world, 172, 56, 286, 86, Material::Methane);

    // Lower collection sump with Sand & Water
    fill_rect(world, 22, 176, 160, HEIGHT - 8, Material::Water);
    fill_rect(world, 166, 172, WIDTH - 22, HEIGHT - 8, Material::Sand);
}

/// 2: Circuit Bomb — Timed burning fuse triggers Spark along a branching Metal
/// and Saltwater conductor network into Gunpowder, Nitro, and Methane bunkers.
fn preset_circuit_bomb(world: &mut World) {
    fill_rect(world, 0, HEIGHT - 8, WIDTH, HEIGHT, Material::Obsidian);

    // Slow burning Wood fuse on far left leading to Gunpowder primer on Metal bus
    fill_rect(world, 18, 102, 42, 105, Material::Wood);
    fill_rect(world, 18, 99, 22, 102, Material::Fire);
    fill_rect(world, 42, 100, 50, 106, Material::Gunpowder);

    // Primary Metal conductor bus
    fill_rect(world, 50, 102, 152, 105, Material::Metal);
    // Upper branch -> Methane + Gunpowder dome
    fill_rect(world, 106, 54, 109, 102, Material::Metal);
    fill_rect(world, 106, 54, 210, 57, Material::Metal);

    // Chamber 1: Upper Methane + Gunpowder
    fill_rect(world, 198, 28, 268, 76, Material::Stone);
    fill_rect(world, 203, 33, 263, 71, Material::Empty);
    fill_rect(world, 208, 54, 258, 71, Material::Gunpowder);
    fill_rect(world, 208, 35, 258, 52, Material::Methane);
    fill_rect(world, 203, 54, 212, 57, Material::Metal);

    // Saltwater liquid bridge in center
    fill_rect(world, 148, 105, 198, 124, Material::Obsidian);
    fill_rect(world, 152, 101, 194, 120, Material::Saltwater);
    fill_rect(world, 194, 106, 246, 109, Material::Metal);

    // Chamber 2: Nitro Core on right
    fill_rect(world, 240, 86, 296, 146, Material::Stone);
    fill_rect(world, 246, 92, 290, 140, Material::Empty);
    fill_rect(world, 248, 114, 288, 140, Material::Nitro);
    fill_rect(world, 240, 106, 254, 109, Material::Metal);

    // Lower branch -> Sand-buried Gunpowder mine
    fill_rect(world, 90, 105, 93, 166, Material::Metal);
    fill_rect(world, 90, 164, 178, 167, Material::Metal);
    fill_rect(world, 174, 154, 206, 176, Material::Gunpowder);
    fill_rect(world, 160, 176, 224, HEIGHT - 8, Material::Sand);
}

/// 3: Rainforest Fire — Lush Wood/Plant canopy, Sand banks, Water pond, Oil slick,
/// and active Fire spreading from the west.
fn preset_rainforest_fire(world: &mut World) {
    // Terrain base
    fill_rect(world, 0, HEIGHT - 16, WIDTH, HEIGHT, Material::Stone);
    fill_rect(world, 0, HEIGHT - 28, 210, HEIGHT - 16, Material::Sand);

    // Right-side lake
    fill_rect(world, 210, HEIGHT - 42, WIDTH, HEIGHT - 16, Material::Water);

    // Oil slick near left hollow
    fill_rect(world, 52, HEIGHT - 32, 92, HEIGHT - 26, Material::Oil);

    // Procedural trees & canopy
    let tree_xs = [36usize, 82, 126, 168, 202];
    for (idx, &tx) in tree_xs.iter().enumerate() {
        let height = 56 + (idx % 3) * 16;
        let top_y = HEIGHT - 28 - height;
        fill_rect(world, tx - 3, top_y, tx + 3, HEIGHT - 28, Material::Wood);
        fill_circle(world, tx as i32, top_y as i32, 18, Material::Plant);
        fill_circle(world, tx as i32 - 10, top_y as i32 + 6, 12, Material::Plant);
        fill_circle(world, tx as i32 + 10, top_y as i32 + 6, 12, Material::Plant);
    }

    // Undergrowth
    for x in (14..206).step_by(6) {
        fill_circle(world, x as i32, (HEIGHT - 30) as i32, 4, Material::Plant);
    }

    // Ignition on the far left tree
    fill_circle(world, 26, (HEIGHT - 74) as i32, 6, Material::Fire);
    fill_rect(world, 22, HEIGHT - 36, 34, HEIGHT - 28, Material::Fire);
}

/// 4: Oil & Acid Tank — Multi-fluid density displacement tank (Oil over Water over
/// Saltwater over Acid/Nitro) with Sand & Ice suspended on destructible Wood shelves.
fn preset_oil_acid_tank(world: &mut World) {
    // Obsidian tank walls
    fill_rect(world, 40, 24, 48, HEIGHT - 12, Material::Obsidian);
    fill_rect(
        world,
        WIDTH - 48,
        24,
        WIDTH - 40,
        HEIGHT - 12,
        Material::Obsidian,
    );
    fill_rect(
        world,
        40,
        HEIGHT - 18,
        WIDTH - 40,
        HEIGHT - 12,
        Material::Obsidian,
    );

    // Intentionally inverted/mixed fluid layers to showcase density sorting
    fill_rect(world, 48, 146, WIDTH - 48, HEIGHT - 18, Material::Oil);
    fill_rect(world, 48, 118, WIDTH - 48, 146, Material::Water);
    fill_rect(world, 48, 92, WIDTH - 48, 118, Material::Saltwater);

    // Wood shelf holding Acid & Sand & Ice above
    fill_rect(world, 48, 72, 150, 76, Material::Wood);
    fill_rect(world, 56, 44, 140, 72, Material::Acid);

    fill_rect(world, 170, 72, WIDTH - 48, 76, Material::Stone);
    fill_rect(world, 180, 40, 258, 72, Material::Sand);
    fill_circle(world, 218, 32, 10, Material::Ice);
}

/// 5: Bio-Hazard Containment — Virus spreading through a Plant/Wood terrarium
/// below an Acid purge reservoir and Water mist chamber.
fn preset_biohazard_lab(world: &mut World) {
    fill_rect(world, 20, 18, WIDTH - 20, 24, Material::Obsidian);
    fill_rect(
        world,
        20,
        HEIGHT - 14,
        WIDTH - 20,
        HEIGHT - 8,
        Material::Obsidian,
    );
    fill_rect(world, 20, 18, 26, HEIGHT - 8, Material::Obsidian);
    fill_rect(
        world,
        WIDTH - 26,
        18,
        WIDTH - 20,
        HEIGHT - 8,
        Material::Obsidian,
    );

    // Upper purge chamber (Acid left, Water right) separated by Wood plugs
    fill_rect(world, 26, 62, WIDTH - 26, 66, Material::Obsidian);
    fill_rect(world, 72, 62, 92, 66, Material::Wood);
    fill_rect(world, 220, 62, 240, 66, Material::Wood);
    fill_rect(world, 26, 30, 148, 62, Material::Acid);
    fill_rect(world, 154, 24, 160, 62, Material::Obsidian);
    fill_rect(world, 160, 30, WIDTH - 26, 62, Material::Water);

    // Terrarium Wood scaffolding and lush Plant lattice
    for y in (94..176).step_by(24) {
        fill_rect(world, 42, y, WIDTH - 42, y + 4, Material::Wood);
        for x in (48..WIDTH - 48).step_by(14) {
            fill_circle(world, x as i32, (y - 6) as i32, 6, Material::Plant);
        }
    }
    fill_rect(world, 156, 86, 162, HEIGHT - 14, Material::Wood);

    // Initial Virus outbreak on lower-left Wood/Plant
    fill_circle(world, 52, 164, 5, Material::Virus);
    fill_circle(world, 96, 140, 4, Material::Virus);
}
