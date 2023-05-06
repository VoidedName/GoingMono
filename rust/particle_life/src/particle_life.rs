use crate::utils::{Colour, Vec2};
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::ops::{Div, Sub};

#[derive(Copy, Clone)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub colour: Colour,
    pub drag: f32,
}

pub struct Force {
    /// Distance the repelling force acts on the particle
    pub d_repel: f32,
    /// Distance the colour based force acts on the particle
    pub d_colour: f32,
    /// Strength of the force. Positive is repelling, Negative is attracting
    pub force: f32,
}

impl Force {
    fn step_colour(&self, x: f32) -> f32 {
        ((x - self.d_repel).signum() - (x - self.d_colour).signum()) / 2.0
    }

    fn step_repel(&self, x: f32) -> f32 {
        (x.signum() - (x - self.d_repel).signum()) / 2.0
    }

    fn curve_colour(&self, x: f32) -> f32 {
        ((x - self.d_repel) * PI / (self.d_colour - self.d_repel)).sin()
    }

    fn force_repel(&self, x: f32) -> f32 {
        (self.d_repel / x) - 1.0
    }

    fn effect(&self, distance: f32) -> f32 {
        let colour = self.step_colour(distance) * self.curve_colour(distance) * self.force;
        let repel = self.step_repel(distance) * self.force_repel(distance);
        colour + repel
    }

    /// Returns a induced force vector by neighbor on the target
    pub fn f(&self, target: &Vec2, neighbor: &Vec2) -> Vec2 {
        // direction pointing at target
        let direction: Vec2 = target - neighbor;
        let mut strength = self.effect(direction.len());
        if !strength.is_finite() {
            strength = self.force
        }
        direction.normalized() * strength.min(0.1)
    }
}

/// Associates the Force acting on the left Particles Colour based on the right ones Colour
pub type ParticleForces = HashMap<(Colour, Colour), Force>;

pub struct World {
    pub size: (u32, u32),
    pub forces: ParticleForces,
    pub particles: Vec<Particle>,
    locations: HashMap<(u32, u32), HashSet<usize>>,
    force_grid_distance: i32,
}

impl World {
    pub fn new(size: (u32, u32), forces: ParticleForces, particles: Vec<Particle>) -> Self {
        let force_grid_distance = forces
            .values()
            .map(|it| it.d_colour)
            .reduce(f32::max)
            .unwrap_or(0.0) as i32
            + 1;
        let mut locations = HashMap::new();

        for x in 0..size.0 as i32 / force_grid_distance {
            for y in 0..size.1 as i32 / force_grid_distance {
                locations.insert((x as u32, y as u32), HashSet::new());
            }
        }

        for (i, particle) in particles.iter().enumerate() {
            locations
                .get_mut(&Self::grid_location(
                    &particle.position,
                    &size,
                    force_grid_distance,
                ))
                .unwrap()
                .insert(i);
        }

        Self {
            size,
            force_grid_distance,
            forces,
            particles,
            locations,
        }
    }

    fn grid_location(position: &Vec2, size: &(u32, u32), grid_size: i32) -> (u32, u32) {
        let x_mod = (size.0 as i32 / grid_size) as u32;
        let y_mod = (size.1 as i32 / grid_size) as u32;
        let pos = position / grid_size as f32;
        let x = (pos.x + x_mod as f32) as u32 % x_mod;
        let y = (pos.y + y_mod as f32) as u32 % y_mod;
        (x, y)
    }

    pub fn simulation_step(&mut self) {
        let x_mod = (self.size.0 as i32 / self.force_grid_distance);
        let y_mod = (self.size.1 as i32 / self.force_grid_distance);

        let mut particles = vec![];
        for (idx1, p1) in self.particles.iter().enumerate() {
            let mut p = p1.clone();
            p.velocity = p.velocity * p.drag;
            p.position = Vec2 {
                x: (p.position.x + p.velocity.x + self.size.0 as f32) % (self.size.0 as f32),
                y: (p.position.y + p.velocity.y + self.size.1 as f32) % (self.size.1 as f32),
            };

            let (grid_x, grid_y) =
                &Self::grid_location(&p.position, &self.size, self.force_grid_distance);

            let mut candidates = HashSet::new();
            for x in -1..=1 {
                for y in -1..=1 {
                    let x = (*grid_x as i32 + x + x_mod) % x_mod;
                    let y = (*grid_y as i32 + y + y_mod) % y_mod;
                    for idx2 in self.locations[&(x as u32, y as u32)].iter() {
                        if *idx2 != idx1 {
                            candidates.insert(*idx2);
                        }
                    }
                }
            }

            for candidate in candidates.iter() {
                let p2 = self.particles[*candidate];
                if let Some(force) = self.forces.get(&(p1.colour, p2.colour)) {
                    p.velocity = p.velocity + force.f(&p.position, &p2.position)
                }
            }

            self.locations
                .get_mut(&Self::grid_location(
                    &p1.position,
                    &self.size,
                    self.force_grid_distance,
                ))
                .unwrap()
                .remove(&idx1);
            self.locations
                .get_mut(&Self::grid_location(
                    &p.position,
                    &self.size,
                    self.force_grid_distance,
                ))
                .unwrap()
                .insert(idx1);

            particles.push(p);
        }

        self.particles = particles;
    }
}
