use rand::distributions::Uniform;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::rand_core::RngCore;
use rand_seeder::Seeder;
use std::cmp::max;
use std::f64::consts::PI;
use std::mem::swap;
use std::process::id;
use std::task::ready;
use vn_engine::color::{BLACK, BLUE, RED, RGBA, WHITE};
use vn_engine::engine::{VNERunner, VNEngineState};
use vn_engine::render::{PixelPosition, VNERenderer};
use vn_engine::Keycode::P;

struct Agent {
    sense_angle: f64,
    sense_distance: f64,
    sense_size: u32,

    speed: f64,
    facing: f64,
    rotation_speed: f64,
    position: (f64, f64),
}

impl Agent {
    fn sense(&self, world: &Grid<Cell>, angle: f64) -> f64 {
        let sense_pos = (
            self.position.0 + (self.facing + angle).cos() * self.sense_distance,
            self.position.1 - (self.facing + angle).sin() * self.sense_distance,
        );
        let sense_center = self.sense_size as f64 / 2.0;
        let mut sense_amount = 0.0;
        for w in 0..self.sense_size {
            for h in 0..self.sense_size {
                let w = (w as f64 - sense_center + sense_pos.0).round();
                let h = (h as f64 - sense_center + sense_pos.1).round();
                if w >= 0.0 && h >= 0.0 && w < world.width as f64 && h < world.height as f64 {
                    let patch = world.xy_index(w as usize, h as usize);
                    sense_amount += world.cells[patch].pheromone;
                }
            }
        }
        sense_amount
    }

    fn turn_angle(&mut self, angle: f64) {
        self.facing = (self.facing + angle) % (2.0*PI);
    }

    fn turn_right(&mut self) {
        self.turn_angle(-self.rotation_speed);
    }

    fn turn_left(&mut self) {
        self.turn_angle(self.rotation_speed);
    }

    fn update_facing(&mut self, rng: &mut Pcg64, world: &Grid<Cell>) {
        let left = self.sense(world, self.sense_angle);
        let front = self.sense(world, 0.0);
        let right = self.sense(world, -self.sense_angle);

        if front > left && front > right {
            return;
        } else if front < right && front < right {
            if rng.gen_bool(0.5) {
                self.turn_left();
            } else {
                self.turn_right();
            };
        } else if left < right {
            self.turn_right()
        } else if right > left {
            self.turn_left();
        }
    }

    fn try_move(&mut self, rng: &mut Pcg64, world: &mut Grid<Cell>) {
        let x = self.position.0 + self.facing.cos() * self.speed;
        let y = self.position.1 - self.facing.sin() * self.speed;

        if x >= 0.0 && y >= 0.0 && x < world.width as f64 && y < world.height as f64 {
            self.position = (x, y);
            let idx = world.xy_index(x as usize, y as usize);
            let cell = &mut world.cells[idx];
            cell.pheromone += 5.0;
        } else {
            self.facing = rng.sample(Uniform::new(0.0, 2.0 * PI))
        }
    }

    fn update(&mut self, rng: &mut Pcg64, world: &mut Grid<Cell>) {
        self.update_facing(rng, world);
        self.try_move(rng, world);
    }
}

#[derive(Copy, Clone)]
struct Cell {
    pheromone: f64,
}

impl Cell {
    fn new() -> Cell {
        Cell { pheromone: 0.0 }
    }
}

struct Grid<T> {
    width: usize,
    height: usize,
    cells: Vec<T>,
}

macro_rules! grid {
    [$new:expr, $width:expr, $height:expr] => {
        Grid {
            width: $width,
            height: $height,
            cells: vec![$new; $width * $height]
        }
    }
}

impl<T> Grid<T> {
    pub fn xy_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

struct Simulation {
    world: Grid<Cell>,
    agents: Vec<Agent>,
    rng: Pcg64,
    bias: Vec<(usize, usize, f64)>
}

impl VNERunner for Simulation {
    fn setup(&mut self, engine: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {
        renderer.set_title("Ant Simulator");
        renderer.fill_rectangle(
            PixelPosition { x: 0, y: 0 },
            PixelPosition {
                x: PIXEL_WIDTH,
                y: PIXEL_HEIGHT,
            },
            WHITE,
        );
    }

    fn tick(&mut self, engine: &VNEngineState, renderer: &mut (impl VNERenderer + ?Sized)) {
        for (x, y, f) in self.bias.iter() {
            let idx = self.world.xy_index(*x, *y);
            self.world.cells[idx].pheromone += *f;
        }

        let mut new_cells = Vec::with_capacity(self.world.width * self.world.height);
        let mut max: f64 = 0.0;
        for y in 0..(self.world.height as i32) {
            for x in 0..(self.world.width as i32) {
                let mut new_value = 0.0;
                for i in -1..=1 {
                    for j in -1..=1 {
                        let nx = x + i;
                        let ny = y + j;
                        if nx >= 0 && ny >= 0 && nx < self.world.width as i32 && ny < self.world.height as i32 {
                            let idx = self.world.xy_index(nx as usize, ny as usize);
                            new_value += self.world.cells[idx].pheromone / 9.0;
                        }
                    }
                }
                max = max.max(new_value);
                let mut new_cell = Cell {
                    pheromone: (new_value - PHEROMONE_EVAPORATION).max(0.0)
                };
                new_cells.push(new_cell);
            }
        }
        self.world.cells = new_cells;

        for x in 0..(self.world.width) {
            for y in 0..(self.world.height) {
                let intensity = (self.world.cells[self.world.xy_index(x, y)].pheromone / max) as f32;
                renderer.fill_rectangle(
                    PixelPosition {
                        x: x as u32 * (PIXEL_WIDTH / self.world.width as u32),
                        y: y as u32 * (PIXEL_HEIGHT / self.world.height as u32),
                    },
                    PixelPosition {
                        x: (1 + x as u32) * (PIXEL_WIDTH / self.world.width as u32) - 1,
                        y: (1 + y as u32) * (PIXEL_HEIGHT / self.world.height as u32) - 1,
                    },
                    RGBA {
                        r: intensity,
                        g: intensity,
                        b: intensity,
                        a: 1.0,
                    },
                );
            }
        }

        for agent in self.agents.iter() {
            let (x, y) = agent.position;
            renderer.draw_pixel(
                PixelPosition {
                    x: (x * (PIXEL_WIDTH / self.world.width as u32) as f64) as u32,
                    y: (y * (PIXEL_HEIGHT / self.world.height as u32) as f64) as u32,
                },
                RED,
            )
        }
        for agent in self.agents.iter_mut() {
            agent.update(&mut self.rng, &mut self.world)
        }
    }
}

const PIXEL_WIDTH: u32 = 500;
const PIXEL_HEIGHT: u32 = 500;
const PHEROMONE_EVAPORATION: f64 = 0.1;

fn main() {
    let width_tiles = PIXEL_WIDTH;
    let height_tiles = PIXEL_WIDTH;
    let mut rng: Pcg64 = Seeder::from("2024").make_rng();

    let mut runner = Simulation {
        world: grid![Cell::new(), width_tiles as usize, height_tiles as usize],
        agents: vec![],
        rng: Seeder::from(rng.gen::<u128>()).make_rng(),
        bias: vec![],
    };

    for _ in 0..10000 {
        runner.agents.push(Agent {
            sense_size: 1,
            sense_distance: 9.0,
            sense_angle: PI / 4.0,
            speed: 1.0,
            facing: rng.sample(Uniform::new(0.0, 2.0 * PI)),

            position: (rng.gen_range(0..width_tiles) as f64, rng.gen_range(0..height_tiles) as f64),

            rotation_speed: PI / 4.0,
        })
    }

    let mut engine = vn_engine::engine::VNEngine::new_sprite_based(PIXEL_WIDTH, PIXEL_HEIGHT, 4);
    engine.run(&mut runner)
}
