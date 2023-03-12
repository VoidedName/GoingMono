use crate::utils::{Colour, Vec2};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::Sub;

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
        direction.normalized() * strength
    }
}

/// Associates the Force acting on the left Particles Colour based on the right ones Colour
pub type ParticleForces = HashMap<(Colour, Colour), Force>;

pub struct World {
    pub size: (u32, u32),
    pub forces: ParticleForces,
    pub particles: Vec<Particle>,
}

impl World {
    pub fn simulation_step(&mut self) {
        self.particles = self
            .particles
            .iter()
            .enumerate()
            .map(|(idx1, p1)| {
                let mut p = p1.clone();
                p.velocity = p.velocity * p.drag;
                for (idx2, p2) in self.particles.iter().enumerate() {
                    if idx2 != idx1 {
                        if let Some(force) = self.forces.get(&(p1.colour, p2.colour)) {
                            p.velocity =
                                p.velocity + force.f(&p.position, &p2.position)
                        }
                    }
                }
                p
            })
            .collect();
    }
}
