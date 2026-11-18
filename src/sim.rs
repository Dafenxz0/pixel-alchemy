//! Cellular pixel physics, multi-fluid density displacement, 2D thermodynamics,
//! electrical BFS conduction, and chemical reaction engine.

use std::collections::VecDeque;

use crate::material::{Material, Phase};

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;
pub const CELL_COUNT: usize = WIDTH * HEIGHT;

pub const MIN_TEMP: f32 = -50.0;
pub const MAX_TEMP: f32 = 1500.0;
pub const AMBIENT_TEMP: f32 = 22.0;

#[derive(Clone, Copy)]
struct Blast {
    cx: usize,
    cy: usize,
    radius: i32,
    shatter_stone: bool,
}

pub struct World {
    pub materials: Vec<u8>,
    pub temps: Vec<f32>,
    temps_next: Vec<f32>,
    pub life: Vec<u8>,
    pub color_var: Vec<u8>,
    updated: Vec<u8>,
    step_stamp: u8,
    pub rgba: Vec<u8>,
    pub tick_count: u32,
    pub reaction_count: u32,
    pub view_mode: u32,
    rng_state: u32,
    bfs_queue: VecDeque<usize>,
    visited: Vec<u32>,
    visit_epoch: u32,
    pending_blasts: Vec<Blast>,
    pub active_count: u32,
    pub phase_counts: [u32; 4],
    pub avg_temp: f32,
    pub max_temp: f32,
}

impl World {
    pub fn new(seed: u32) -> Self {
        let mut world = Self {
            materials: vec![Material::Empty as u8; CELL_COUNT],
            temps: vec![AMBIENT_TEMP; CELL_COUNT],
            temps_next: vec![AMBIENT_TEMP; CELL_COUNT],
            life: vec![0; CELL_COUNT],
            color_var: vec![0; CELL_COUNT],
            updated: vec![0; CELL_COUNT],
            step_stamp: 1,
            rgba: vec![0; CELL_COUNT * 4],
            tick_count: 0,
            reaction_count: 0,
            view_mode: 0,
            rng_state: if seed == 0 { 0x9E37_79B9 } else { seed },
            bfs_queue: VecDeque::with_capacity(1024),
            visited: vec![0; CELL_COUNT],
            visit_epoch: 0,
            pending_blasts: Vec::with_capacity(32),
            active_count: 0,
            phase_counts: [0; 4],
            avg_temp: AMBIENT_TEMP,
            max_temp: AMBIENT_TEMP,
        };
        world.render();
        world
    }

    pub fn clear(&mut self) {
        self.materials.fill(Material::Empty as u8);
        self.temps.fill(AMBIENT_TEMP);
        self.temps_next.fill(AMBIENT_TEMP);
        self.life.fill(0);
        self.color_var.fill(0);
        self.updated.fill(0);
        self.step_stamp = 1;
        self.tick_count = 0;
        self.reaction_count = 0;
        self.pending_blasts.clear();
        self.render();
    }

    #[inline]
    pub fn idx(x: usize, y: usize) -> usize {
        y * WIDTH + x
    }

    #[inline]
    fn rand_u32(&mut self) -> u32 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_state = x;
        x
    }

    #[inline]
    fn rand_chance(&mut self, num: u32, den: u32) -> bool {
        (self.rand_u32() % den) < num
    }

    #[inline]
    pub fn material_at(&self, x: i32, y: i32) -> Material {
        if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
            return Material::Empty;
        }
        Material::from_u8(self.materials[Self::idx(x as usize, y as usize)])
    }

    #[inline]
    pub fn temp_at(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
            return AMBIENT_TEMP;
        }
        self.temps[Self::idx(x as usize, y as usize)]
    }

    pub fn set_cell(&mut self, x: usize, y: usize, mat: Material) {
        if x >= WIDTH || y >= HEIGHT {
            return;
        }
        let i = Self::idx(x, y);
        self.materials[i] = mat as u8;
        self.temps[i] = mat.props().default_temp;
        self.color_var[i] = (self.rand_u32() & 0xFF) as u8;
        self.life[i] = match mat {
            Material::Fire => 22 + (self.rand_u32() % 22) as u8,
            Material::Spark => 5 + (self.rand_u32() % 5) as u8,
            Material::Smoke => 110 + (self.rand_u32() % 70) as u8,
            Material::ToxicGas => 180 + (self.rand_u32() % 70) as u8,
            _ => 0,
        };
    }

    pub fn paint(&mut self, cx: i32, cy: i32, radius: i32, mat: Material) {
        let r = radius.clamp(1, 24);
        let r2 = r * r;
        let scattered = matches!(
            mat.phase(),
            Phase::Powder | Phase::Liquid | Phase::Gas | Phase::Energy
        ) && r > 1;

        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r2 {
                    continue;
                }
                let px = cx + dx;
                let py = cy + dy;
                if px < 0 || py < 0 || px >= WIDTH as i32 || py >= HEIGHT as i32 {
                    continue;
                }
                if scattered && mat != Material::Empty && self.rand_chance(1, 3) {
                    continue;
                }
                let idx = Self::idx(px as usize, py as usize);
                if mat == Material::Empty
                    || self.materials[idx] == Material::Empty as u8
                    || mat.phase() == Phase::Solid
                    || mat == Material::Fire
                    || mat == Material::Spark
                {
                    self.set_cell(px as usize, py as usize, mat);
                }
            }
        }
    }

    pub fn paint_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, radius: i32, mat: Material) {
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.paint(x, y, radius, mat);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn thermal_brush(&mut self, cx: i32, cy: i32, radius: i32, delta_temp: f32) {
        let r = radius.clamp(1, 24);
        let r2 = r * r;
        for dy in -r..=r {
            for dx in -r..=r {
                let d2 = dx * dx + dy * dy;
                if d2 > r2 {
                    continue;
                }
                let px = cx + dx;
                let py = cy + dy;
                if px < 0 || py < 0 || px >= WIDTH as i32 || py >= HEIGHT as i32 {
                    continue;
                }
                let idx = Self::idx(px as usize, py as usize);
                let falloff = 1.0 - (d2 as f32 / (r2 as f32 + 1.0)) * 0.5;
                let next_t = self.temps[idx] + delta_temp * falloff;
                self.temps[idx] = next_t.clamp(MIN_TEMP, MAX_TEMP);
            }
        }
    }

    pub fn step(&mut self, steps: u32) {
        for _ in 0..steps {
            self.step_once();
        }
        self.render();
    }

    fn next_stamp(&mut self) {
        self.step_stamp = self.step_stamp.wrapping_add(1);
        if self.step_stamp == 0 {
            self.updated.fill(0);
            self.step_stamp = 1;
        }
    }

    fn step_once(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        self.next_stamp();
        let left_to_right = (self.tick_count & 1) == 0;

        // Pass 1: Electrical BFS conduction along Metal / Saltwater / Water
        self.propagate_electricity();
        self.process_blasts();

        // Pass 2: Bottom-to-top for Powders, Liquids, Solids, Energy, and Reactions
        for y in (0..HEIGHT).rev() {
            for xi in 0..WIDTH {
                let x = if left_to_right { xi } else { WIDTH - 1 - xi };
                let idx = Self::idx(x, y);
                if self.updated[idx] == self.step_stamp {
                    continue;
                }
                let mat = Material::from_u8(self.materials[idx]);
                match mat.phase() {
                    Phase::Void | Phase::Gas => {}
                    Phase::Powder => self.update_powder(x, y, idx, mat),
                    Phase::Liquid => self.update_liquid(x, y, idx, mat),
                    Phase::Solid => self.update_solid(x, y, idx, mat),
                    Phase::Energy => self.update_energy(x, y, idx, mat),
                }
            }
        }

        // Pass 3: Top-to-bottom for rising Gases
        for y in 0..HEIGHT {
            for xi in 0..WIDTH {
                let x = if left_to_right { xi } else { WIDTH - 1 - xi };
                let idx = Self::idx(x, y);
                if self.updated[idx] == self.step_stamp {
                    continue;
                }
                let mat = Material::from_u8(self.materials[idx]);
                if mat.phase() == Phase::Gas {
                    self.update_gas(x, y, idx, mat);
                }
            }
        }

        // Pass 4: Iterative explosions queued during movement
        self.process_blasts();

        // Pass 5: 2D Thermal Diffusion & Phase Transitions
        self.step_thermodynamics();
        self.process_blasts();
    }

    #[inline]
    fn swap_cells(&mut self, a: usize, b: usize) {
        self.materials.swap(a, b);
        self.temps.swap(a, b);
        self.life.swap(a, b);
        self.color_var.swap(a, b);
        self.updated[a] = self.step_stamp;
        self.updated[b] = self.step_stamp;
    }

    #[inline]
    fn can_displace(mover: Material, target: Material) -> bool {
        if target == Material::Empty || target.phase() == Phase::Gas {
            return true;
        }
        target.phase() == Phase::Liquid && mover.density() > target.density()
    }

    fn update_powder(&mut self, x: usize, y: usize, idx: usize, mat: Material) {
        if mat == Material::Virus {
            self.react_virus(x, y, idx);
            if self.materials[idx] != Material::Virus as u8 {
                return;
            }
        }

        if y + 1 < HEIGHT {
            let below = Self::idx(x, y + 1);
            let below_mat = Material::from_u8(self.materials[below]);
            if Self::can_displace(mat, below_mat) {
                self.swap_cells(idx, below);
                return;
            }

            let dir: isize = if self.rand_chance(1, 2) { 1 } else { -1 };
            for d in [dir, -dir] {
                let nx = x as isize + d;
                if nx >= 0 && nx < WIDTH as isize {
                    let diag = Self::idx(nx as usize, y + 1);
                    let diag_mat = Material::from_u8(self.materials[diag]);
                    if Self::can_displace(mat, diag_mat) {
                        self.swap_cells(idx, diag);
                        return;
                    }
                }
            }
        }
    }

    fn update_liquid(&mut self, x: usize, y: usize, idx: usize, mat: Material) {
        if self.life[idx] > 0 {
            self.life[idx] -= 1;
        }

        if mat == Material::Acid {
            self.react_acid(x, y, idx);
            if self.materials[idx] != Material::Acid as u8 {
                return;
            }
        } else if mat == Material::Nitro && self.check_nitro_trigger(x, y, idx) {
            return;
        }

        // Buoyancy helper: if a denser liquid sits directly above us, swap upward promptly
        if y > 0 {
            let above = Self::idx(x, y - 1);
            let above_mat = Material::from_u8(self.materials[above]);
            if above_mat.phase() == Phase::Liquid && above_mat.density() > mat.density() {
                self.swap_cells(idx, above);
                return;
            }
        }

        // Fall straight down
        if y + 1 < HEIGHT {
            let below = Self::idx(x, y + 1);
            let below_mat = Material::from_u8(self.materials[below]);
            if Self::can_displace(mat, below_mat) {
                self.swap_cells(idx, below);
                return;
            }

            // Diagonal fall
            let dir: isize = if self.rand_chance(1, 2) { 1 } else { -1 };
            for d in [dir, -dir] {
                let nx = x as isize + d;
                if nx >= 0 && nx < WIDTH as isize {
                    let diag = Self::idx(nx as usize, y + 1);
                    let diag_mat = Material::from_u8(self.materials[diag]);
                    if Self::can_displace(mat, diag_mat) {
                        self.swap_cells(idx, diag);
                        return;
                    }
                }
            }
        }

        // Horizontal dispersion
        let disp = mat.props().dispersion as isize;
        let dir: isize = if self.rand_chance(1, 2) { 1 } else { -1 };
        for d in [dir, -dir] {
            let mut best = None;
            for step in 1..=disp {
                let nx = x as isize + d * step;
                if nx < 0 || nx >= WIDTH as isize {
                    break;
                }
                let side = Self::idx(nx as usize, y);
                let side_mat = Material::from_u8(self.materials[side]);
                if side_mat == Material::Empty || side_mat.phase() == Phase::Gas {
                    best = Some(side);
                } else if side_mat.phase() == Phase::Liquid && mat.density() > side_mat.density() {
                    best = Some(side);
                    break;
                } else {
                    break;
                }
            }
            if let Some(target) = best {
                self.swap_cells(idx, target);
                return;
            }
        }
    }

    fn update_gas(&mut self, x: usize, y: usize, idx: usize, mat: Material) {
        if mat == Material::Smoke || mat == Material::ToxicGas {
            if self.life[idx] > 0 {
                self.life[idx] -= 1;
            }
            if self.life[idx] == 0 {
                self.set_cell(x, y, Material::Empty);
                return;
            }
        }

        if mat == Material::Methane && self.has_ignition_neighbor(x, y) {
            self.queue_blast(x, y, 4, false);
            self.reaction_count = self.reaction_count.wrapping_add(1);
            return;
        }

        // Rise straight up or diagonally up with turbulence
        if y > 0 {
            let above = Self::idx(x, y - 1);
            let above_mat = Material::from_u8(self.materials[above]);
            if (above_mat == Material::Empty
                || (above_mat.phase() == Phase::Gas && mat.density() < above_mat.density()))
                && self.rand_chance(4, 5)
            {
                self.swap_cells(idx, above);
                return;
            }

            let dir: isize = if self.rand_chance(1, 2) { 1 } else { -1 };
            for d in [dir, -dir] {
                let nx = x as isize + d;
                if nx >= 0 && nx < WIDTH as isize {
                    let diag = Self::idx(nx as usize, y - 1);
                    let diag_mat = Material::from_u8(self.materials[diag]);
                    if diag_mat == Material::Empty
                        || (diag_mat.phase() == Phase::Gas && mat.density() < diag_mat.density())
                    {
                        self.swap_cells(idx, diag);
                        return;
                    }
                }
            }
        }

        // Lateral gas dispersion
        let disp = mat.props().dispersion as isize;
        let dir: isize = if self.rand_chance(1, 2) { 1 } else { -1 };
        for d in [dir, -dir] {
            for step in 1..=disp {
                let nx = x as isize + d * step;
                if nx < 0 || nx >= WIDTH as isize {
                    break;
                }
                let side = Self::idx(nx as usize, y);
                let side_mat = Material::from_u8(self.materials[side]);
                if side_mat == Material::Empty {
                    if step == disp || self.rand_chance(1, 2) {
                        self.swap_cells(idx, side);
                        return;
                    }
                } else {
                    break;
                }
            }
        }
    }

    fn update_solid(&mut self, x: usize, y: usize, idx: usize, mat: Material) {
        if self.life[idx] > 0 {
            self.life[idx] -= 1;
        }
        if mat == Material::Plant {
            self.react_plant(x, y, idx);
        }
    }

    fn update_energy(&mut self, x: usize, y: usize, idx: usize, mat: Material) {
        if mat == Material::Fire {
            // Extinguished by Water/Saltwater/Ice/Steam
            for (nx, ny) in self.neighbors4(x, y) {
                let n_idx = Self::idx(nx, ny);
                let n_mat = Material::from_u8(self.materials[n_idx]);
                if matches!(n_mat, Material::Water | Material::Saltwater | Material::Ice) {
                    self.set_cell(x, y, Material::Smoke);
                    self.temps[idx] = 110.0;
                    self.reaction_count = self.reaction_count.wrapping_add(1);
                    return;
                }
                // Ignite neighbors
                if n_mat.props().flammable {
                    let chance = match n_mat {
                        Material::Gunpowder | Material::Nitro | Material::Methane => 255,
                        Material::Oil => 50,
                        Material::Plant => 40,
                        Material::Wood => 14,
                        Material::Virus => 95,
                        _ => 20,
                    };
                    if self.rand_chance(chance, 255) {
                        self.ignite_cell(nx, ny, n_idx, n_mat);
                    }
                }
            }

            if self.life[idx] > 0 {
                self.life[idx] -= 1;
            }
            if self.life[idx] == 0 {
                if self.rand_chance(2, 5) {
                    self.set_cell(x, y, Material::Smoke);
                    self.temps[idx] = 135.0;
                } else {
                    self.set_cell(x, y, Material::Empty);
                }
                return;
            }

            // Fire flickers upward
            if y > 0 && self.rand_chance(2, 3) {
                let dx = (self.rand_u32() % 3) as isize - 1;
                let nx = x as isize + dx;
                if nx >= 0 && nx < WIDTH as isize {
                    let target = Self::idx(nx as usize, y - 1);
                    let t_mat = Material::from_u8(self.materials[target]);
                    if t_mat == Material::Empty || t_mat == Material::Smoke {
                        self.swap_cells(idx, target);
                    }
                }
            }
        } else if mat == Material::Spark {
            // Spark triggers explosives on contact
            for (nx, ny) in self.neighbors4(x, y) {
                let n_idx = Self::idx(nx, ny);
                let n_mat = Material::from_u8(self.materials[n_idx]);
                if matches!(
                    n_mat,
                    Material::Gunpowder | Material::Nitro | Material::Methane | Material::Oil
                ) {
                    self.ignite_cell(nx, ny, n_idx, n_mat);
                }
            }

            if self.life[idx] > 0 {
                self.life[idx] -= 1;
            }
            if self.life[idx] == 0 {
                self.set_cell(x, y, Material::Empty);
                return;
            }

            // Jittery arc step
            let dx = (self.rand_u32() % 3) as isize - 1;
            let dy = (self.rand_u32() % 3) as isize - 1;
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && nx < WIDTH as isize && ny >= 0 && ny < HEIGHT as isize {
                let target = Self::idx(nx as usize, ny as usize);
                if self.materials[target] == Material::Empty as u8 {
                    self.swap_cells(idx, target);
                }
            }
        }
    }

    fn react_acid(&mut self, x: usize, y: usize, idx: usize) {
        let mut dirs = [(0isize, 1isize), (1, 0), (-1, 0), (0, -1)];
        if self.rand_chance(1, 2) {
            dirs.swap(1, 2);
        }
        for (dx, dy) in dirs {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 || nx >= WIDTH as isize || ny >= HEIGHT as isize {
                continue;
            }
            let n_idx = Self::idx(nx as usize, ny as usize);
            let n_mat = Material::from_u8(self.materials[n_idx]);
            let sus = n_mat.props().acid_dissolve;
            if sus > 0 && self.rand_chance(sus as u32, 255) {
                if self.rand_chance(3, 5) {
                    self.set_cell(nx as usize, ny as usize, Material::ToxicGas);
                    self.temps[n_idx] = (self.temps[n_idx] + 24.0).min(MAX_TEMP);
                } else {
                    self.set_cell(nx as usize, ny as usize, Material::Empty);
                }
                self.temps[idx] = (self.temps[idx] + 20.0).min(MAX_TEMP);
                self.reaction_count = self.reaction_count.wrapping_add(1);
                if self.rand_chance(2, 5) {
                    if self.rand_chance(1, 2) {
                        self.set_cell(x, y, Material::ToxicGas);
                    } else {
                        self.set_cell(x, y, Material::Empty);
                    }
                }
                return;
            }
        }
    }

    fn react_plant(&mut self, x: usize, y: usize, idx: usize) {
        let t = self.temps[idx];
        if !(2.0..=55.0).contains(&t) {
            return;
        }
        for (nx, ny) in self.neighbors4(x, y) {
            let n_idx = Self::idx(nx, ny);
            if self.materials[n_idx] == Material::Water as u8 && self.rand_chance(1, 14) {
                // Consume water to grow plant
                self.set_cell(nx, ny, Material::Plant);
                self.reaction_count = self.reaction_count.wrapping_add(1);
                return;
            }
        }
    }

    fn react_virus(&mut self, x: usize, y: usize, idx: usize) {
        if self.temps[idx] >= 120.0 {
            self.set_cell(x, y, Material::Smoke);
            self.reaction_count = self.reaction_count.wrapping_add(1);
            return;
        }
        for (nx, ny) in self.neighbors4(x, y) {
            let n_idx = Self::idx(nx, ny);
            let n_mat = Material::from_u8(self.materials[n_idx]);
            if matches!(n_mat, Material::Fire | Material::Lava | Material::Acid) {
                self.set_cell(x, y, Material::Empty);
                self.reaction_count = self.reaction_count.wrapping_add(1);
                return;
            }
            if matches!(n_mat, Material::Plant | Material::Wood) && self.rand_chance(1, 7) {
                self.set_cell(nx, ny, Material::Virus);
                self.reaction_count = self.reaction_count.wrapping_add(1);
                return;
            }
        }
    }

    fn check_nitro_trigger(&mut self, x: usize, y: usize, idx: usize) -> bool {
        if self.temps[idx] >= 110.0 {
            self.queue_blast(x, y, 11, true);
            return true;
        }
        for (nx, ny) in self.neighbors4(x, y) {
            let n_mat = Material::from_u8(self.materials[Self::idx(nx, ny)]);
            if matches!(
                n_mat,
                Material::Fire | Material::Spark | Material::Lava | Material::Acid
            ) {
                self.queue_blast(x, y, 11, true);
                return true;
            }
        }
        false
    }

    fn has_ignition_neighbor(&self, x: usize, y: usize) -> bool {
        for (nx, ny) in self.neighbors4(x, y) {
            let n_mat = Material::from_u8(self.materials[Self::idx(nx, ny)]);
            if matches!(n_mat, Material::Fire | Material::Spark | Material::Lava) {
                return true;
            }
        }
        false
    }

    fn ignite_cell(&mut self, x: usize, y: usize, idx: usize, mat: Material) {
        match mat {
            Material::Gunpowder => {
                self.queue_blast(x, y, 6, false);
                self.reaction_count = self.reaction_count.wrapping_add(1);
            }
            Material::Nitro => {
                self.queue_blast(x, y, 11, true);
                self.reaction_count = self.reaction_count.wrapping_add(1);
            }
            Material::Methane => {
                self.queue_blast(x, y, 4, false);
                self.reaction_count = self.reaction_count.wrapping_add(1);
            }
            Material::Oil | Material::Plant | Material::Wood => {
                self.set_cell(x, y, Material::Fire);
                self.temps[idx] = 680.0;
                self.reaction_count = self.reaction_count.wrapping_add(1);
            }
            Material::Virus => {
                self.set_cell(x, y, Material::Smoke);
                self.temps[idx] = 220.0;
                self.reaction_count = self.reaction_count.wrapping_add(1);
            }
            _ => {}
        }
    }

    fn queue_blast(&mut self, cx: usize, cy: usize, radius: i32, shatter_stone: bool) {
        if self.pending_blasts.len() < 28 {
            self.pending_blasts.push(Blast {
                cx,
                cy,
                radius,
                shatter_stone,
            });
        }
    }

    fn process_blasts(&mut self) {
        let mut cursor = 0;
        while cursor < self.pending_blasts.len() && cursor < 28 {
            let blast = self.pending_blasts[cursor];
            cursor += 1;
            self.execute_blast(blast);
        }
        self.pending_blasts.clear();
    }

    fn execute_blast(&mut self, blast: Blast) {
        let r = blast.radius;
        let r2 = r * r;
        let inner_r2 = (r * r) / 3;
        let core_r2 = (r * r) / 5;

        for dy in -r..=r {
            for dx in -r..=r {
                let d2 = dx * dx + dy * dy;
                if d2 > r2 {
                    continue;
                }
                let px = blast.cx as isize + dx as isize;
                let py = blast.cy as isize + dy as isize;
                if px < 0 || py < 0 || px >= WIDTH as isize || py >= HEIGHT as isize {
                    continue;
                }
                let ux = px as usize;
                let uy = py as usize;
                let idx = Self::idx(ux, uy);
                let mat = Material::from_u8(self.materials[idx]);

                // Thermal shockwave
                let heat = if d2 <= inner_r2 { 820.0 } else { 320.0 };
                self.temps[idx] = (self.temps[idx] + heat).min(MAX_TEMP);

                // Chain detonation of adjacent explosives
                if matches!(mat, Material::Gunpowder | Material::Nitro | Material::Methane)
                    && !(ux == blast.cx && uy == blast.cy)
                {
                    let sub_r = match mat {
                        Material::Nitro => 10,
                        Material::Gunpowder => 6,
                        _ => 4,
                    };
                    self.queue_blast(ux, uy, sub_r, mat == Material::Nitro);
                    self.set_cell(ux, uy, Material::Fire);
                    self.reaction_count = self.reaction_count.wrapping_add(1);
                    continue;
                }

                if mat == Material::Obsidian {
                    continue;
                }

                let destroy = if mat == Material::Stone || mat == Material::Metal {
                    blast.shatter_stone && d2 <= core_r2
                } else if mat.phase() == Phase::Solid {
                    d2 <= inner_r2
                } else {
                    d2 <= inner_r2 || self.rand_chance(2, 5)
                };

                if destroy {
                    if d2 <= core_r2 || self.rand_chance(1, 2) {
                        if self.rand_chance(1, 5) {
                            self.set_cell(ux, uy, Material::Spark);
                        } else {
                            self.set_cell(ux, uy, Material::Fire);
                        }
                        self.temps[idx] = 950.0;
                    } else {
                        self.set_cell(ux, uy, Material::Smoke);
                        self.temps[idx] = 420.0;
                    }
                } else if mat == Material::Empty && self.rand_chance(1, 3) {
                    self.set_cell(ux, uy, Material::Smoke);
                    self.temps[idx] = 280.0;
                } else if mat.phase() == Phase::Powder || mat.phase() == Phase::Liquid {
                    // Radial shrapnel knockback
                    let max_x = WIDTH as isize - 1;
                    let max_y = HEIGHT as isize - 1;
                    let sx = (px + dx.signum() as isize * 2).clamp(0, max_x) as usize;
                    let sy = (py + dy.signum() as isize * 2 - 1).clamp(0, max_y) as usize;
                    let target = Self::idx(sx, sy);
                    if self.materials[target] == Material::Empty as u8 {
                        self.swap_cells(idx, target);
                    }
                }
            }
        }
    }

    fn propagate_electricity(&mut self) {
        self.visit_epoch = self.visit_epoch.wrapping_add(1);
        if self.visit_epoch == 0 {
            self.visited.fill(0);
            self.visit_epoch = 1;
        }
        let epoch = self.visit_epoch;
        self.bfs_queue.clear();

        // Seed BFS from every active Spark
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = Self::idx(x, y);
                if self.materials[idx] == Material::Spark as u8 {
                    for (nx, ny) in self.neighbors4(x, y) {
                        let n_idx = Self::idx(nx, ny);
                        let n_mat = Material::from_u8(self.materials[n_idx]);
                        if n_mat.props().conductive
                            && n_mat != Material::Spark
                            && self.life[n_idx] == 0
                            && self.visited[n_idx] != epoch
                        {
                            self.visited[n_idx] = epoch;
                            self.bfs_queue.push_back(n_idx);
                        }
                    }
                }
            }
        }

        let mut metal_steps = 0usize;
        let mut fluid_steps = 0usize;
        while let Some(idx) = self.bfs_queue.pop_front() {
            let mat = Material::from_u8(self.materials[idx]);
            if mat == Material::Metal {
                if metal_steps >= 1400 {
                    continue;
                }
                metal_steps += 1;
            } else {
                if fluid_steps >= 180 {
                    continue;
                }
                fluid_steps += 1;
            }

            // Electrify cell and heat slightly
            self.life[idx] = 10;
            self.temps[idx] = (self.temps[idx] + 35.0).min(MAX_TEMP);
            self.reaction_count = self.reaction_count.wrapping_add(1);

            let x = idx % WIDTH;
            let y = idx / WIDTH;
            let mut conductor_neighbors = 0u8;

            for (nx, ny) in self.neighbors4(x, y) {
                let n_idx = Self::idx(nx, ny);
                let n_mat = Material::from_u8(self.materials[n_idx]);
                if matches!(
                    n_mat,
                    Material::Gunpowder | Material::Nitro | Material::Methane | Material::Oil
                ) {
                    self.ignite_cell(nx, ny, n_idx, n_mat);
                    continue;
                }
                if n_mat.props().conductive && n_mat != Material::Spark {
                    conductor_neighbors += 1;
                    let allow = match n_mat {
                        Material::Metal => true,
                        Material::Saltwater => fluid_steps < 160,
                        Material::Water | Material::Acid => fluid_steps < 36,
                        _ => false,
                    };
                    if allow && self.life[n_idx] == 0 && self.visited[n_idx] != epoch {
                        self.visited[n_idx] = epoch;
                        self.bfs_queue.push_back(n_idx);
                    }
                }
            }

            // Emit visible sparks from wire tips into Empty space
            if mat == Material::Metal && conductor_neighbors <= 1 {
                for (nx, ny) in self.neighbors4(x, y) {
                    let n_idx = Self::idx(nx, ny);
                    if self.materials[n_idx] == Material::Empty as u8 && self.rand_chance(1, 5) {
                        self.set_cell(nx, ny, Material::Spark);
                        break;
                    }
                }
            }
        }
    }

    fn step_thermodynamics(&mut self) {
        // 1. Source/Sink thermal adjustments and quench heat exchange
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = Self::idx(x, y);
                let mat = Material::from_u8(self.materials[idx]);
                let mut t = self.temps[idx];

                match mat {
                    Material::Lava => {
                        let mut quench_contacts = 0i32;
                        for (nx, ny) in self.neighbors4(x, y) {
                            let n_mat = Material::from_u8(self.materials[Self::idx(nx, ny)]);
                            if matches!(
                                n_mat,
                                Material::Water | Material::Saltwater | Material::Ice
                            ) {
                                quench_contacts += 1;
                            }
                        }
                        if quench_contacts > 0 {
                            t -= 65.0 * quench_contacts as f32;
                        } else {
                            t += (1150.0 - t) * 0.035;
                        }
                    }
                    Material::Fire => {
                        t += (650.0 - t) * 0.15;
                    }
                    Material::Steam => {
                        if y < HEIGHT / 4 {
                            t -= 1.05;
                        } else if y < HEIGHT / 2 {
                            t -= 0.45;
                        } else {
                            t -= 0.12;
                        }
                    }
                    Material::Smoke => {
                        t += (35.0 - t) * 0.03;
                    }
                    Material::Empty if y < 6 => {
                        // Upper-atmosphere cooling boundary so rising Steam condenses into rain
                        t += (12.0 - t) * 0.08;
                    }
                    _ => {}
                }

                self.temps[idx] = t.clamp(MIN_TEMP, MAX_TEMP);
            }
        }

        // 2. 2D 4-neighbor thermal diffusion
        for y in 0..HEIGHT {
            let row = y * WIDTH;
            for x in 0..WIDTH {
                let idx = row + x;
                let t = self.temps[idx];
                let cond = Material::from_u8(self.materials[idx]).props().conductivity;
                let mut flux = 0.0f32;

                if x > 0 {
                    let nb = idx - 1;
                    let k = cond.min(Material::from_u8(self.materials[nb]).props().conductivity);
                    flux += k * 0.45 * (self.temps[nb] - t);
                }
                if x + 1 < WIDTH {
                    let nb = idx + 1;
                    let k = cond.min(Material::from_u8(self.materials[nb]).props().conductivity);
                    flux += k * 0.45 * (self.temps[nb] - t);
                }
                if y > 0 {
                    let nb = idx - WIDTH;
                    let k = cond.min(Material::from_u8(self.materials[nb]).props().conductivity);
                    flux += k * 0.45 * (self.temps[nb] - t);
                }
                if y + 1 < HEIGHT {
                    let nb = idx + WIDTH;
                    let k = cond.min(Material::from_u8(self.materials[nb]).props().conductivity);
                    flux += k * 0.45 * (self.temps[nb] - t);
                }

                self.temps_next[idx] = (t + flux).clamp(MIN_TEMP, MAX_TEMP);
            }
        }
        std::mem::swap(&mut self.temps, &mut self.temps_next);

        // 3. Phase changes & auto-ignition
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = Self::idx(x, y);
                let mat = Material::from_u8(self.materials[idx]);
                let t = self.temps[idx];

                match mat {
                    Material::Water => {
                        if t >= 100.0 {
                            self.set_cell(x, y, Material::Steam);
                            self.temps[idx] = t.max(108.0);
                            self.reaction_count = self.reaction_count.wrapping_add(1);
                        } else if t <= 0.0 {
                            self.set_cell(x, y, Material::Ice);
                            self.temps[idx] = t.min(-2.0);
                            self.reaction_count = self.reaction_count.wrapping_add(1);
                        }
                    }
                    Material::Saltwater => {
                        if t >= 106.0 {
                            self.set_cell(x, y, Material::Steam);
                            self.temps[idx] = t.max(112.0);
                            self.reaction_count = self.reaction_count.wrapping_add(1);
                        } else if t <= -12.0 {
                            self.set_cell(x, y, Material::Ice);
                            self.temps[idx] = t.min(-14.0);
                            self.reaction_count = self.reaction_count.wrapping_add(1);
                        }
                    }
                    Material::Steam => {
                        if t <= 82.0 {
                            self.set_cell(x, y, Material::Water);
                            self.temps[idx] = 42.0;
                            self.reaction_count = self.reaction_count.wrapping_add(1);
                        }
                    }
                    Material::Ice => {
                        if t >= 2.0 {
                            self.set_cell(x, y, Material::Water);
                            self.temps[idx] = 8.0;
                            self.reaction_count = self.reaction_count.wrapping_add(1);
                        }
                    }
                    Material::Lava => {
                        if t <= 660.0 {
                            let mut water_quench = false;
                            for (nx, ny) in self.neighbors4(x, y) {
                                let n_mat = Material::from_u8(self.materials[Self::idx(nx, ny)]);
                                if matches!(
                                    n_mat,
                                    Material::Water
                                        | Material::Saltwater
                                        | Material::Ice
                                        | Material::Steam
                                ) {
                                    water_quench = true;
                                    break;
                                }
                            }
                            let solid = if water_quench {
                                Material::Obsidian
                            } else {
                                Material::Stone
                            };
                            self.set_cell(x, y, solid);
                            self.temps[idx] = t;
                            self.reaction_count = self.reaction_count.wrapping_add(1);
                        }
                    }
                    Material::Acid => {
                        if t >= 145.0 {
                            self.set_cell(x, y, Material::ToxicGas);
                            self.temps[idx] = t;
                            self.reaction_count = self.reaction_count.wrapping_add(1);
                        }
                    }
                    _ => {
                        let props = mat.props();
                        if props.flammable && t >= props.ignition_temp {
                            self.ignite_cell(x, y, idx, mat);
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn neighbors4(&self, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
        let mut out = [(0usize, 0usize); 4];
        let mut count = 0usize;
        if y + 1 < HEIGHT {
            out[count] = (x, y + 1);
            count += 1;
        }
        if x > 0 {
            out[count] = (x - 1, y);
            count += 1;
        }
        if x + 1 < WIDTH {
            out[count] = (x + 1, y);
            count += 1;
        }
        if y > 0 {
            out[count] = (x, y - 1);
            count += 1;
        }
        out.into_iter().take(count)
    }

    pub fn render(&mut self) {
        let mut active = 0u32;
        let mut phases = [0u32; 4];
        let mut temp_sum = 0.0f64;
        let mut max_t = MIN_TEMP;

        for i in 0..CELL_COUNT {
            let mat = Material::from_u8(self.materials[i]);
            let t = self.temps[i];
            temp_sum += t as f64;
            if t > max_t {
                max_t = t;
            }

            match mat.phase() {
                Phase::Void => {}
                Phase::Powder => {
                    active += 1;
                    phases[0] += 1;
                }
                Phase::Liquid => {
                    active += 1;
                    phases[1] += 1;
                }
                Phase::Gas => {
                    active += 1;
                    phases[2] += 1;
                }
                Phase::Solid | Phase::Energy => {
                    active += 1;
                    phases[3] += 1;
                }
            }

            let [r, g, b] = if self.view_mode == 1 {
                thermal_palette(t)
            } else {
                material_pixel_color(mat, t, self.life[i], self.color_var[i], self.tick_count)
            };

            let o = i * 4;
            self.rgba[o] = r;
            self.rgba[o + 1] = g;
            self.rgba[o + 2] = b;
            self.rgba[o + 3] = 255;
        }

        self.active_count = active;
        self.phase_counts = phases;
        self.avg_temp = (temp_sum / CELL_COUNT as f64) as f32;
        self.max_temp = max_t;
    }
}

fn material_pixel_color(mat: Material, temp: f32, life: u8, var: u8, tick: u32) -> [u8; 3] {
    let props = mat.props();
    let [mut r, mut g, mut b] = props.base_color;

    if props.color_jitter > 0 {
        let j = props.color_jitter as i32;
        let offset = (var as i32 % (j * 2 + 1)) - j;
        r = (r as i32 + offset).clamp(0, 255) as u8;
        g = (g as i32 + offset).clamp(0, 255) as u8;
        b = (b as i32 + offset).clamp(0, 255) as u8;
    }

    match mat {
        Material::Empty => {
            // Subtle thermal shimmer in heated air
            if temp > 90.0 {
                let glow = ((temp - 90.0) / 500.0).clamp(0.0, 1.0);
                r = (r as f32 + glow * 45.0) as u8;
                g = (g as f32 + glow * 14.0) as u8;
            } else if temp < 0.0 {
                let chill = (-temp / 50.0).clamp(0.0, 1.0);
                b = (b as f32 + chill * 26.0) as u8;
            }
        }
        Material::Lava => {
            let pulse = (((tick.wrapping_add(var as u32)) % 24) as f32 / 24.0 - 0.5).abs();
            let hot = ((temp - 650.0) / 550.0).clamp(0.0, 1.0);
            r = (210.0 + hot * 45.0) as u8;
            g = (52.0 + hot * 110.0 + pulse * 40.0).min(255.0) as u8;
            b = (12.0 + hot * 36.0) as u8;
        }
        Material::Fire => {
            let f = (life as f32 / 44.0).clamp(0.0, 1.0);
            r = 255;
            g = (90.0 + f * 155.0).min(255.0) as u8;
            b = (15.0 + f * 90.0).min(255.0) as u8;
        }
        Material::Spark => {
            r = 210;
            g = 248;
            b = 255;
        }
        Material::Metal | Material::Water | Material::Saltwater if life > 0 => {
            let z = (life as f32 / 10.0).clamp(0.0, 1.0);
            r = (r as f32 + (170.0 - r as f32) * z * 0.7) as u8;
            g = (g as f32 + (245.0 - g as f32) * z * 0.9) as u8;
            b = (b as f32 + (255.0 - b as f32) * z) as u8;
        }
        _ => {
            // Subtle incandescent glow on hot solids/powders
            if temp > 350.0 && mat != Material::Obsidian {
                let glow = ((temp - 350.0) / 800.0).clamp(0.0, 0.7);
                r = (r as f32 + (255.0 - r as f32) * glow) as u8;
                g = (g as f32 + (110.0 - g as f32) * glow * 0.5) as u8;
            }
        }
    }

    [r, g, b]
}

fn thermal_palette(temp: f32) -> [u8; 3] {
    // -50C (cryo blue) -> 22C (dark navy) -> 120C (amber) -> 600C (red-orange) -> 1500C (white-hot)
    if temp < 22.0 {
        let t = ((temp - MIN_TEMP) / (22.0 - MIN_TEMP)).clamp(0.0, 1.0);
        let r = (36.0 * (1.0 - t) + 14.0 * t) as u8;
        let g = (170.0 * (1.0 - t) + 22.0 * t) as u8;
        let b = (255.0 * (1.0 - t) + 48.0 * t) as u8;
        [r, g, b]
    } else if temp < 220.0 {
        let t = ((temp - 22.0) / (220.0 - 22.0)).clamp(0.0, 1.0);
        let r = (14.0 + t * 210.0) as u8;
        let g = (22.0 + t * 90.0) as u8;
        let b = (48.0 * (1.0 - t) + 20.0 * t) as u8;
        [r, g, b]
    } else if temp < 750.0 {
        let t = ((temp - 220.0) / (750.0 - 220.0)).clamp(0.0, 1.0);
        let r = (224.0 + t * 31.0) as u8;
        let g = (112.0 + t * 110.0) as u8;
        let b = (20.0 + t * 40.0) as u8;
        [r, g, b]
    } else {
        let t = ((temp - 750.0) / (MAX_TEMP - 750.0)).clamp(0.0, 1.0);
        let r = 255;
        let g = (222.0 + t * 33.0) as u8;
        let b = (60.0 + t * 195.0) as u8;
        [r, g, b]
    }
}
