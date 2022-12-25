use std::f64::consts::PI;
use std::mem::swap;
use std::ops::Index;
use std::process::Output;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use vn_engine::color::{BLACK, BLUE, GREEN, RED, RGBA, WHITE, YELLOW};
use vn_engine::engine::{EngineState, VNERunner};
use vn_engine::render::{PixelPosition, VNERenderer};
use crate::BufferUse::{FIRST, SECOND};
use crate::SnowFlakeGridData::{BOUNDARY, EDGE, FROZEN, NOT_RECEPTIVE};

enum BufferUse {
    FIRST,
    SECOND,
}

struct Runner {
    rand: ThreadRng,
    snow_flake_buffer1: SnowFlake,
    snow_flake_buffer2: SnowFlake,
    buffer_in_use: BufferUse,
    iterations: u32,
    max_iterations: u32,
    stop: bool,
    settings: Vec<SimulationSetting>,
}

impl Runner {
    fn new(size: u32, settings: Vec<SimulationSetting>, cluster: bool) -> Runner {
        let buffer1 = SnowFlake::new(size, settings[0].beta, cluster);
        let buffer2 = SnowFlake::new(size, settings[0].beta, cluster);
        Runner {
            rand: rand::thread_rng(),
            snow_flake_buffer1: buffer1,
            snow_flake_buffer2: buffer2,
            buffer_in_use: FIRST,
            iterations: 0,
            max_iterations: settings.iter().fold(0, |a, b| a + b.iterations),
            stop: false,
            settings,
        }
    }

    /// (current, buffer)
    pub fn snowflake_data(&mut self) -> (&SnowFlake, &mut SnowFlake) {
        match &self.buffer_in_use {
            FIRST => (&self.snow_flake_buffer1, &mut self.snow_flake_buffer2),
            SECOND => (&self.snow_flake_buffer2, &mut self.snow_flake_buffer1),
        }
    }

    fn snowflake(&mut self) -> &SnowFlake {
        match &self.buffer_in_use {
            FIRST => &self.snow_flake_buffer1,
            SECOND => &self.snow_flake_buffer2,
        }
    }

    fn snowflake_buffer(&mut self) -> &mut SnowFlake {
        match &self.buffer_in_use {
            FIRST => &mut self.snow_flake_buffer2,
            SECOND => &mut self.snow_flake_buffer1,
        }
    }

    pub fn swap_buffers(&mut self) {
        match &self.buffer_in_use {
            FIRST => self.buffer_in_use = SECOND,
            SECOND => self.buffer_in_use = FIRST,
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
    pub fn new<F>(cols: u32, rows: u32, init: F) -> HexGrid<T> where F: Fn(u32, u32) -> T {
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

    pub fn hex_space(x: u32, y: u32) -> (i32, i32) {
        (x as i32 - (y / 2) as i32, x as i32 + ((y as i32 + 1) / 2))
    }

    pub fn distance(p1: (u32, u32), p2: (u32, u32)) -> u32 {
        let (x1, y1) = HexGrid::<SnowFlakeGridData>::hex_space(p1.0, p1.1);
        let (x2, y2) = HexGrid::<SnowFlakeGridData>::hex_space(p2.0, p2.1);
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

impl<T> HexGrid<T> {
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
}

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
    pub fn simulate_step<F>(&self, target: &mut SnowFlake, mut rules: F)
        where F: FnMut(&SnowFlakeGridData, &HexGrid<SnowFlakeGridData>, &HexNeighbors) -> SnowFlakeGridData
    {
        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                if let Some(state) = self.grid.get(x, y) {
                    let new_data = rules(state, &self.grid, &self.grid.neighbors(x, y));
                    target.grid.set(x, y, new_data);
                }
            }
        }
    }
}

impl SnowFlake {
    pub fn new(resolution: u32, background: f64, cluster: bool) -> SnowFlake {
        let mut rng: Pcg64 = Seeder::from("2023").make_rng();
        let seeds = rng.gen_range(2..4);
        let mut seed_locations = Vec::new();
        if cluster {
            for seed in 0..seeds {
                seed_locations.push((rng.gen_range(resolution - 5..resolution + 5), rng.gen_range(resolution - 5..resolution + 5)));
            }
        } else {
            seed_locations.push((resolution, resolution));
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
    fn setup(&mut self, _engine: &EngineState, renderer: &mut impl VNERenderer) {
        renderer.set_title("Flaky SnowFlake Generator!");
    }

    fn tick(&mut self, _engine: &EngineState, renderer: &mut impl VNERenderer) {
        let mut iterations = self.iterations;
        let mut setting_nr = 0;
        for setting in self.settings.iter() {
            if iterations >= setting.iterations {
                iterations -= setting.iterations;
                setting_nr += 1;
            } else {
                break;
            }
        }

        if setting_nr >= self.settings.len() {
            self.stop = true;
        }

        let mut stop = self.stop.clone();
        if self.iterations < self.max_iterations && !self.stop {
            let mut settings = &self.settings[setting_nr];
            let alpha = settings.alpha;
            let beta = settings.beta;
            let gamma = settings.gamma;

            let fallback = EDGE { moisture: beta };

            self.iterations += 1;
            if self.iterations % 10 == 0 {
                renderer.set_title(format!("Flaky SnowFlake Generator! Setting: {}; Iteration: {};", setting_nr, self.iterations).as_str());
            }
            let (snowflake, buffer) = self.snowflake_data();
            {
                snowflake.simulate_step(buffer, |data, grid, neighbors| {
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
                                stop = true;
                            }
                        }
                    }

                    data.updated(&nn, alpha, beta, gamma)
                });
            }
            self.stop = stop;
            self.swap_buffers();
        }

        renderer.clear_screen(BLACK);

        for y in 0..self.snowflake().resolution * 2 + 1 {
            for x in 0..self.snowflake().resolution * 2 + 1 {
                if let Some(state) = self.snowflake().grid.get(x, y) {
                    match state {
                        BOUNDARY { moisture } | EDGE { moisture } | NOT_RECEPTIVE { moisture } => {
                            if !stop {
                                let intensity = (*moisture as f32 * 0.25).min(0.25);
                                self.fill_hex((x, y), 4.0, RGBA { r: 0.51 * intensity, g: intensity * 0.8, b: intensity, a: 1.0 }, renderer);
                            }
                        }
                        FROZEN { moisture } => {
                            let intensity = 1.0 / (*moisture as f32);
                            self.fill_hex((x, y), 4.0, RGBA { r: intensity, g: intensity, b: intensity, a: 1.0 }, renderer);
                        }
                    }
                }
            }
        }
    }
}


/// Simulation is controlled by the settings.
/// It stops when either the snowflake reaches the edge or the last setting ran out of iterations.
///
/// Simulation is based on a cellular automaton.
///  - cells are hexagons
///  - a cell freezes once its water content reaches 1
///  - a cell either diffuses water or absorbs water.
///  - frozen cells or those adjacent to frozen ones absorb water.
///  - all other cells diffuse water
///
/// alpha: controls diffusion across the plane (i.e. how fast the water travels between cells)
/// beta: controls how fast water is injected from the border
/// gamma: constant background vapour
/// main reference i used https://www.patarnott.com/pdf/SnowCrystalGrowth.pdf
fn main() {
    let settings = vec![
        SimulationSetting { alpha: 2.0, beta: 0.5, gamma: 0.0001, iterations: 250 },
        SimulationSetting { alpha: 1.0, beta: 0.1, gamma: 0.1, iterations: 30 },
        SimulationSetting { alpha: 1.0, beta: 0.8, gamma: 0.01, iterations: 70 },
        SimulationSetting { alpha: 1.0, beta: 0.3, gamma: 0.000001, iterations: 300 },
    ];
    let mut engine = vn_engine::engine::VNEngine::new_opengl(1165, 1000, 2);
    let mut runner = Runner::new(28 * 3, settings, false);
    engine.run(&mut runner);
}
