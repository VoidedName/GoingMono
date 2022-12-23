use std::f64::consts::PI;
use std::mem::swap;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use vn_engine::color::{BLACK, BLUE, GREEN, RED, RGBA, WHITE, YELLOW};
use vn_engine::engine::VNERunner;
use vn_engine::render::{PixelPosition, VNERenderer};
use crate::SnowFlakeGridData::{BOUNDARY, EDGE, FROZEN, NOT_RECEPTIVE};

struct Runner {
    rand: ThreadRng,
    snow_flake: SnowFlake,
    time_since_last_update: u128,
    iterations: u32,
    max_iterations: u32,
    stop: bool,
    settings: Vec<SimulationSetting>,
}

impl Runner {
    fn new(size: u32, settings: Vec<SimulationSetting>) -> Runner {
        Runner {
            rand: rand::thread_rng(),
            snow_flake: SnowFlake::new(size, settings[0].beta),
            time_since_last_update: 0,
            iterations: 0,
            max_iterations: 10_000_000,
            stop: false,
            settings,
        }
    }
}

struct SimulationSetting {
    alpha: f64,
    beta: f64,
    gamma: f64,
    iterations: u32,
}

#[derive(Debug)]
struct HexNeighbors {
    left: Option<(u32, u32)>,
    top_left: Option<(u32, u32)>,
    top_right: Option<(u32, u32)>,
    right: Option<(u32, u32)>,
    bottom_right: Option<(u32, u32)>,
    bottom_left: Option<(u32, u32)>,
}

// every other row is shifted
// 0   1   2   3   4
//   6   7   8   9   10
// 11 ...
struct HexGrid<CellData> {
    rows: u32,
    cols: u32,
    cells: Vec<CellData>,
}

impl<T> HexGrid<T> {
    pub fn new<D, F>(cols: u32, rows: u32, init: F) -> HexGrid<D> where F: Fn(u32, u32) -> D {
        let mut cells = Vec::with_capacity(rows as usize * cols as usize);
        for y in 0..rows {
            for x in 0..cols {
                cells.push(init(x, y));
            }
        }
        HexGrid {
            rows,
            cols,
            cells,
        }
    }

    fn idx(&self, col: u32, row: u32) -> usize {
        ((row * self.cols) + col) as usize
    }

    pub fn get(&self, col: u32, row: u32) -> Option<&T> {
        if row >= self.rows || col >= self.cols { return None; }
        let idx = self.idx(col, row);
        Some(&self.cells[idx])
    }

    pub fn set(&mut self, col: u32, row: u32, data: T) {
        if row >= self.rows || col >= self.cols { return; }
        let idx = self.idx(col, row);
        self.cells[idx] = data;
    }

    pub fn hex_space(x: u32, y: u32) -> (i32, i32) {
        (x as i32 - (y / 2) as i32, x as i32 + ((y as i32 + 1) / 2))
    }

    pub fn distance(p1: (u32, u32), p2: (u32, u32)) -> u32 {
        let (x1, y1) = HexGrid::<T>::hex_space(p1.0, p1.1);
        let (x2, y2) = HexGrid::<T>::hex_space(p2.0, p2.1);
        let dx = x2 - x1;
        let dy = y2 - y1;
        if dx.signum() == dy.signum() {
            dx.abs().max(dy.abs()) as u32
        } else {
            (dx.abs() + dy.abs()) as u32
        }
    }

    pub fn neighbors(&self, col: u32, row: u32) -> HexNeighbors {
        let is_offset_row = (row % 2) == 1;
        let is_top = row == 0;
        let is_bottom = row == self.rows - 1;
        let is_left = col == 0;
        let is_right = col == self.cols - 1;

        let left = if is_left { None } else { Some((col - 1, row)) };
        let right = if is_right { None } else { Some((col + 1, row)) };

        let top_left = if is_top || (is_left && !is_offset_row) {
            None
        } else {
            Some((
                if is_offset_row { col } else { col - 1 },
                row - 1,
            ))
        };

        let top_right = if is_top || (is_right && is_offset_row) {
            None
        } else {
            Some((
                if is_offset_row { col + 1 } else { col },
                row - 1,
            ))
        };

        let bottom_left = if is_bottom || (is_left && !is_offset_row) {
            None
        } else {
            Some((
                if is_offset_row { col } else { col - 1 },
                row + 1,
            ))
        };

        let bottom_right = if is_bottom || (is_right && is_offset_row) {
            None
        } else {
            Some((
                if is_offset_row { col + 1 } else { col },
                row + 1,
            ))
        };

        HexNeighbors {
            left,
            top_left,
            top_right,
            right,
            bottom_right,
            bottom_left,
        }
    }
}

// https://www.patarnott.com/pdf/SnowCrystalGrowth.pdf
// moisture > 0.0 is considered frozen
// BOUNDARY and FROZEN are receptive cells
// BORDER and NOT_RECEPTIVE are not receptive cells
// alpha is diffusion inside the system
// gamma is water addition through vapour
#[derive(PartialEq, Copy, Clone)]
enum SnowFlakeGridData {
    BOUNDARY {
        moisture: f64, // s_t(z)
    },
    FROZEN {
        moisture: f64, // s_t(z)
    },
    NOT_RECEPTIVE {
        moisture: f64, // s_t(z)
    },
    EDGE {
        moisture: f64, // s_t(z)
    },
}

impl SnowFlakeGridData {
    pub fn diffusing_moisture(&self) -> f64 { // u_t
        match self {
            FROZEN { .. } | BOUNDARY { .. } => 0.0,
            NOT_RECEPTIVE { moisture, .. } | EDGE { moisture, .. } => *moisture,
        }
    }

    pub fn stable_moisture(&self) -> f64 { // v_t
        match self {
            FROZEN { moisture, .. } | BOUNDARY { moisture, .. } => *moisture,
            NOT_RECEPTIVE { .. } | EDGE { .. } => 0.0,
        }
    }

    pub fn updated(&self, neighbors: &[&SnowFlakeGridData; 6], alpha: f64, beta: f64, gamma: f64) -> SnowFlakeGridData {
        let stable = match self {
            FROZEN { .. } | BOUNDARY { .. } => self.stable_moisture() + gamma,
            _ => self.stable_moisture(),
        };

        const TWO_OVER_THREE: f64 = 2.0 / 3.0;

        let mut sum_nn = 0.0;
        let mut has_frozen_nn = false;
        for x in neighbors {
            sum_nn += x.diffusing_moisture();
            if let FROZEN { .. } = **x {
                has_frozen_nn = true;
            }
        }

        let diffusion = self.diffusing_moisture() + (alpha / 12.0) * (-6.0 * self.diffusing_moisture() + sum_nn);
        // let diffusion = self.diffusing_moisture() + (alpha / 2.0) * (sum_nn / 6.0 - self.diffusing_moisture());

        let moisture = diffusion + stable;

        match self {
            FROZEN { .. } => {
                FROZEN { moisture }
            }
            BOUNDARY { .. } => {
                if moisture >= 1.0 {
                    FROZEN { moisture }
                } else {
                    BOUNDARY { moisture }
                }
            }
            NOT_RECEPTIVE { .. } => {
                if has_frozen_nn {
                    BOUNDARY { moisture }
                } else {
                    NOT_RECEPTIVE { moisture: moisture.min(1.0).max(0.0) }
                }
            }
            EDGE { .. } => {
                EDGE { moisture: beta }
            }
        }
    }
}

// hex grid is row based
// every other row is offset by half a hex
// hex grid size is based on distance from center?
struct SnowFlake {
    resolution: u32,
    grid: HexGrid<SnowFlakeGridData>,
}

impl SnowFlake {
    pub fn simulate_step<F>(&mut self, mut rules: F)
        where F: FnMut(&SnowFlakeGridData, &HexGrid<SnowFlakeGridData>, &HexNeighbors) -> SnowFlakeGridData
    {
        let mut new_grid_data: Vec<SnowFlakeGridData> = Vec::new();
        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                if let Some(state) = self.grid.get(x, y) {
                    let new_data = rules(state, &self.grid, &self.grid.neighbors(x, y));
                    new_grid_data.push(new_data)
                }
            }
        }
        self.grid.cells = new_grid_data;
    }
}

impl SnowFlake {
    pub fn new(resolution: u32, background: f64) -> SnowFlake {
        let mut rng: Pcg64 = Seeder::from("2022").make_rng();
        let seeds = rng.gen_range(1..8);
        let mut seed_locations = Vec::new();
        for seed in 0..seeds {
            seed_locations.push((rng.gen_range(resolution-5..resolution+5), rng.gen_range(resolution-5..resolution+5)));
        }

        SnowFlake {
            resolution,
            grid: HexGrid::<SnowFlakeGridData>::new(resolution * 2 + 1, resolution * 2 + 1, |x, y| {
                let distance = HexGrid::<SnowFlakeGridData>::distance((x, y), (resolution, resolution));
                if seed_locations.contains(&(x, y)) {
                    FROZEN { moisture: 1.0 }
                } else if distance < resolution {
                    NOT_RECEPTIVE { moisture: background }
                } else {
                    EDGE { moisture: background }
                }
            }),
        }
    }
}

fn rotate(p1: (f64, f64), angle: f64) -> (f64, f64) {
    let (x, y) = p1;
    let (sin, cos) = angle.sin_cos();
    (x * cos - y * sin,
     y * cos + x * sin)
}

impl Runner {
    fn compute_hex_vertices(&self, pos: (u32, u32), scale: f64) -> [PixelPosition; 6] {
        const ONE_SIXTH: f64 = PI / 3.0;

        let t_offset: (f64, f64) = (0.0, scale);
        let tl_offset: (f64, f64) = rotate(t_offset, ONE_SIXTH);
        let bl_offset: (f64, f64) = rotate(tl_offset, ONE_SIXTH);
        let b_offset: (f64, f64) = rotate(bl_offset, ONE_SIXTH);
        let br_offset: (f64, f64) = rotate(b_offset, ONE_SIXTH);
        let tr_offset: (f64, f64) = rotate(br_offset, ONE_SIXTH);

        let (x, y) = pos;

        let x_offset = if y % 2 == 0 {
            0.0
        } else {
            scale * (3.0_f64).sqrt() * 0.5
        };

        let x = (x as f64 * scale * (3.0_f64).sqrt() + x_offset);
        let y = (y as f64 * scale * 0.75 * 2.0);

        let t = PixelPosition {
            x: (x + t_offset.0).round() as u32,
            y: (y + t_offset.1).round() as u32,
        };

        let tr = PixelPosition {
            x: (x + tr_offset.0).round() as u32,
            y: (y + tr_offset.1).round() as u32,
        };

        let br = PixelPosition {
            x: (x + br_offset.0).round() as u32,
            y: (y + br_offset.1).round() as u32,
        };

        let b = PixelPosition {
            x: (x + b_offset.0).round() as u32,
            y: (y + b_offset.1).round() as u32,
        };

        let bl = PixelPosition {
            x: (x + bl_offset.0).round() as u32,
            y: (y + bl_offset.1).round() as u32,
        };

        let tl = PixelPosition {
            x: (x + tl_offset.0).round() as u32,
            y: (y + tl_offset.1).round() as u32,
        };

        [t, tr, br, b, bl, tl]
    }

    fn fill_hex(&self, pos: (u32, u32), scale: f64, color: RGBA, renderer: &mut impl VNERenderer) {
        let [t, tr, br, b, bl, tl] = self.compute_hex_vertices(pos, scale);

        renderer.fill_triangle(tl, t, tr, color);
        renderer.fill_triangle(tl, tr, br, color);
        renderer.fill_triangle(tl, bl, br, color);
        renderer.fill_triangle(bl, b, br, color);
    }

    fn draw_hex(&self, pos: (u32, u32), scale: f64, color: RGBA, renderer: &mut impl VNERenderer) {
        let [t, tr, br, b, bl, tl] = self.compute_hex_vertices(pos, scale);

        renderer.draw_line(t, tl, color);
        renderer.draw_line(tl, bl, color);
        renderer.draw_line(bl, b, color);
        renderer.draw_line(b, br, color);
        renderer.draw_line(br, tr, color);
        renderer.draw_line(tr, t, color);
    }
}

impl VNERunner for Runner {
    fn setup(&mut self, renderer: &mut impl VNERenderer) {
        renderer.set_title("Flaky SnowFlake Generator!");
    }

    fn tick(&mut self, nano_delta: u128, renderer: &mut impl VNERenderer) {
        self.time_since_last_update += nano_delta;

        let mut settings = &self.settings[0];
        let mut iterations = self.iterations;
        for setting in self.settings.iter() {
            while iterations >= setting.iterations {
                iterations -= setting.iterations;
                settings = setting;
            }
        }

        let alpha = settings.alpha;
        let beta = settings.beta;
        let gamma = settings.gamma;

        if self.time_since_last_update >= 1 {
            self.time_since_last_update = 0;

            let fallback = EDGE { moisture: beta };

            if self.iterations < self.max_iterations && !self.stop {
                self.iterations += 1;
                self.snow_flake.simulate_step(|data, grid, neighbors| {
                    let a = *neighbors.left.map(|(x, y)| grid.get(x, y).unwrap()).get_or_insert(&fallback);
                    let b = *neighbors.top_left.map(|(x, y)| grid.get(x, y).unwrap()).get_or_insert(&fallback);
                    let c = *neighbors.top_right.map(|(x, y)| grid.get(x, y).unwrap()).get_or_insert(&fallback);
                    let f = *neighbors.right.map(|(x, y)| grid.get(x, y).unwrap()).get_or_insert(&fallback);
                    let d = *neighbors.bottom_right.map(|(x, y)| grid.get(x, y).unwrap()).get_or_insert(&fallback);
                    let e = *neighbors.bottom_left.map(|(x, y)| grid.get(x, y).unwrap()).get_or_insert(&fallback);

                    let nn = [a, b, c, d, e, f];

                    if let FROZEN { .. } = data {
                        for n in nn {
                            if let EDGE { .. } = n {
                                self.stop = true;
                            }
                        }
                    }

                    data.updated(&nn, alpha, beta, gamma)
                });
            }
        }

        // renderer.clear_screen(BLACK);

        for y in 0..self.snow_flake.resolution * 2 + 1 {
            for x in 0..self.snow_flake.resolution * 2 + 1 {
                if let Some(state) = self.snow_flake.grid.get(x, y) {
                    match state {
                        BOUNDARY { moisture } | EDGE { moisture } | NOT_RECEPTIVE { moisture } => {
                            let intensity = (*moisture as f32 * 0.2).min(0.2);
                            self.fill_hex((x, y), 2.0, RGBA { r: 0.51 * intensity, g: intensity * 0.8, b: intensity, a: 1.0 }, renderer);
                        }
                        FROZEN { moisture } => {
                            let intensity = 1.0 / *moisture as f32;
                            self.fill_hex((x, y), 2.0, RGBA { r: intensity, g: intensity, b: intensity, a: 1.0 }, renderer);
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let alpha = 0.5;
    let beta = 0.2;
    let gamma = 0.05;
    let settings = vec![
        SimulationSetting { alpha: 2.69, beta: 0.3, gamma: 0.002, iterations: 10 },
        SimulationSetting { alpha: 1.0, beta: 0.4, gamma: 0.01, iterations: 10 },
        SimulationSetting { alpha: 2.03, beta: 0.3, gamma: 0.002, iterations: 1 },

        // SimulationSetting { alpha: 2.003, beta: 0.9, gamma: 0.0001, iterations: 1000 },
        // SimulationSetting { alpha: 2.003, beta: 0.03, gamma: 0.00005, iterations: 1000 },
    ];
    let mut engine = vn_engine::engine::VNEngine::new_opengl(775 * 2, 680 * 2, 1);
    let mut runner = Runner::new(28 * 8, settings);
    engine.run(&mut runner);
}
