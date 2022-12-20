use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::f32::consts::PI;
use std::ops;
use std::process::Output;
use vn_engine::color::{BLACK, RGBA, WHITE};
use vn_engine::engine::{VNERunner, VNEngine};
use vn_engine::opengl::OpenGLRenderer;
use vn_engine::render::{Position, VNERenderer};

#[derive(Debug, Copy, Clone)]
struct Vec3D {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3D {
    pub fn cross_product(&self, other: &Vec3D) -> Vec3D {
        Vec3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn dot_product(&self, other: &Vec3D) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn normalize(&mut self) {
        let l = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        self.x /= l;
        self.y /= l;
        self.z /= l;
    }
}

impl ops::Sub for Vec3D {
    type Output = Vec3D;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Add for Vec3D {
    type Output = Vec3D;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Triangle {
    p: [Vec3D; 3],
}

#[derive(Debug, Clone)]
struct Mesh {
    tris: Vec<Triangle>,
}

#[derive(Debug, Copy, Clone)]
struct Matrix4x4 {
    // [row][col]
    m: [[f32; 4]; 4],
}

struct Rasterizer {
    width: u32,
    height: u32,
    rng: ThreadRng,
    time_since_last_fps_draw: u128,
    frames_since_last_fps_draw: u128,

    elapsed: f32,

    mesh_cube: Mesh,
    projection: Matrix4x4,
    camera: Vec3D,
}

impl Rasterizer {
    pub fn new(width: u32, height: u32) -> Rasterizer {
        let aspect_ration = height as f32 / width as f32;
        let field_of_view = PI / 2.0;
        let fov_scale = 1.0 / (field_of_view / 2.0).tan();
        let z_far = 1000.0;
        let z_near = 0.1;

        Rasterizer {
            width,
            height,
            elapsed: 0.0,
            rng: rand::thread_rng(),
            time_since_last_fps_draw: 0,
            frames_since_last_fps_draw: 0,
            #[rustfmt::skip]
            projection: Matrix4x4 {
                m: [[
                    aspect_ration * fov_scale, 0.0, 0.0, 0.0,
                ], [
                    0.0, fov_scale, 0.0, 0.0,
                ], [
                    0.0, 0.0, z_far / (z_far - z_near), 1.0,
                ], [
                    0.0, 0.0, (-z_far * z_near) / (z_far - z_near), 0.0,
                ]]
            },
            #[rustfmt::skip]
            mesh_cube: Mesh {
                tris: vec![
                    // SOUTH
                    Triangle { p: [Vec3D { x: 0.0, y: 0.0, z: 0.0 }, Vec3D { x: 0.0, y: 1.0, z: 0.0 },  Vec3D { x: 1.0, y: 1.0, z: 0.0 }] },
                    Triangle { p: [Vec3D { x: 0.0, y: 0.0, z: 0.0 }, Vec3D { x: 1.0, y: 1.0, z: 0.0 },  Vec3D { x: 1.0, y: 0.0, z: 0.0 }] },

                    // EAST
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 0.0 }, Vec3D { x: 1.0, y: 1.0, z: 0.0 },  Vec3D { x: 1.0, y: 1.0, z: 1.0 }] },
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 0.0 }, Vec3D { x: 1.0, y: 1.0, z: 1.0 },  Vec3D { x: 1.0, y: 0.0, z: 1.0 }] },

                    // NORTH
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 1.0 }, Vec3D { x: 1.0, y: 1.0, z: 1.0 },  Vec3D { x: 0.0, y: 1.0, z: 1.0 }] },
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 1.0, z: 1.0 },  Vec3D { x: 0.0, y: 0.0, z: 1.0 }] },

                    // WEST
                    Triangle { p: [Vec3D { x: 0.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 1.0, z: 1.0 },  Vec3D { x: 0.0, y: 1.0, z: 0.0 }] },
                    Triangle { p: [Vec3D { x: 0.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 1.0, z: 0.0 },  Vec3D { x: 0.0, y: 0.0, z: 0.0 }] },

                    // TOP
                    Triangle { p: [Vec3D { x: 0.0, y: 1.0, z: 0.0 }, Vec3D { x: 0.0, y: 1.0, z: 1.0 },  Vec3D { x: 1.0, y: 1.0, z: 1.0 }] },
                    Triangle { p: [Vec3D { x: 0.0, y: 1.0, z: 0.0 }, Vec3D { x: 1.0, y: 1.0, z: 1.0 },  Vec3D { x: 1.0, y: 1.0, z: 0.0 }] },

                    // BOTTOM
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 0.0, z: 1.0 },  Vec3D { x: 0.0, y: 0.0, z: 0.0 }] },
                    Triangle { p: [Vec3D { x: 1.0, y: 0.0, z: 1.0 }, Vec3D { x: 0.0, y: 0.0, z: 0.0 },  Vec3D { x: 1.0, y: 0.0, z: 0.0 }] },
                ]
            },
            camera: Vec3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }

    fn update_fps(&mut self, delta_nano: u128, renderer: &mut impl VNERenderer) {
        self.time_since_last_fps_draw += delta_nano;
        self.frames_since_last_fps_draw += 1;
        if self.time_since_last_fps_draw > 200_000_000 {
            let fps =
                (self.frames_since_last_fps_draw * 1_000_000_000) / self.time_since_last_fps_draw;
            self.time_since_last_fps_draw = 0;
            self.frames_since_last_fps_draw = 0;
            renderer.set_title(format!("Test - FPS: {}", fps).as_str());
        }
    }

    fn multiply_vector_matrix(vec: &Vec3D, mat: &Matrix4x4) -> Vec3D {
        let mut x =
            (vec.x * mat.m[0][0]) + (vec.y * mat.m[1][0]) + (vec.z * mat.m[2][0]) + mat.m[3][0];
        let mut y =
            (vec.x * mat.m[0][1]) + (vec.y * mat.m[1][1]) + (vec.z * mat.m[2][1]) + mat.m[3][1];
        let mut z =
            (vec.x * mat.m[0][2]) + (vec.y * mat.m[1][2]) + (vec.z * mat.m[2][2]) + mat.m[3][2];
        let w = (vec.x * mat.m[0][3]) + (vec.y * mat.m[1][3]) + (vec.z * mat.m[2][3]) + mat.m[3][3];

        if w != 0.0 {
            x /= w;
            y /= w;
            z /= w;
        }

        Vec3D { x, y, z }
    }
}

impl VNERunner for Rasterizer {
    fn tick(&mut self, delta_nano: u128, renderer: &mut impl VNERenderer) {
        self.update_fps(delta_nano, renderer);

        self.elapsed += delta_nano as f32 / 1_000_000_000 as f32;

        renderer.clear_screen(BLACK);

        let theta = 1.0 * self.elapsed;

        #[rustfmt::skip]
        let rot_z = Matrix4x4 {
            m: [[
                theta.cos(), theta.sin(), 0.0, 0.0,
            ], [
                -theta.sin(), theta.cos(), 0.0, 0.0,
            ], [
                0.0, 0.0, 1.0, 0.0,
            ], [
                0.0, 0.0, 0.0, 1.0,
            ]]
        };

        #[rustfmt::skip]
        let rot_x = Matrix4x4 {
            m: [[
                1.0, 0.0, 0.0, 0.0,
            ], [
                0.0, (theta*0.5).cos(), (theta*0.5).sin(), 0.0,
            ], [
                0.0, -(theta*0.5).sin(), (theta*0.5).cos(), 0.0,
            ], [
                0.0, 0.0, 0.0, 1.0,
            ]]
        };

        // Draw Triangles
        // screen space normalized left/right x=-1 / 1; bottom/top y=-1 / 1; front/back z=-1 / 1
        for tri in self.mesh_cube.tris.iter() {
            let mut tri_rotated_z = tri.clone();
            tri_rotated_z.p[0] = Rasterizer::multiply_vector_matrix(&tri.p[0], &rot_z);
            tri_rotated_z.p[1] = Rasterizer::multiply_vector_matrix(&tri.p[1], &rot_z);
            tri_rotated_z.p[2] = Rasterizer::multiply_vector_matrix(&tri.p[2], &rot_z);

            let mut tri_rotated_x = tri.clone();
            tri_rotated_x.p[0] = Rasterizer::multiply_vector_matrix(&tri_rotated_z.p[0], &rot_x);
            tri_rotated_x.p[1] = Rasterizer::multiply_vector_matrix(&tri_rotated_z.p[1], &rot_x);
            tri_rotated_x.p[2] = Rasterizer::multiply_vector_matrix(&tri_rotated_z.p[2], &rot_x);

            let mut tri_translated = tri_rotated_x.clone();
            tri_translated.p[0].z += 3.0;
            tri_translated.p[1].z += 3.0;
            tri_translated.p[2].z += 3.0;

            let line1 = tri_translated.p[1] - tri_translated.p[0];
            let line2 = tri_translated.p[2] - tri_translated.p[0];
            let mut normal = line1.cross_product(&line2);
            normal.normalize();

            let similarity = normal.dot_product(&(tri_translated.p[0] - self.camera));

            if similarity < 0.0 {
                let mut v1 =
                    Rasterizer::multiply_vector_matrix(&tri_translated.p[0], &self.projection);
                let mut v2 =
                    Rasterizer::multiply_vector_matrix(&tri_translated.p[1], &self.projection);
                let mut v3 =
                    Rasterizer::multiply_vector_matrix(&tri_translated.p[2], &self.projection);

                v1.x += 1.0;
                v1.y += 1.0;

                v2.x += 1.0;
                v2.y += 1.0;

                v3.x += 1.0;
                v3.y += 1.0;

                v1.x *= 0.5 * self.width as f32;
                v1.y *= 0.5 * self.height as f32;

                v2.x *= 0.5 * self.width as f32;
                v2.y *= 0.5 * self.height as f32;

                v3.x *= 0.5 * self.width as f32;
                v3.y *= 0.5 * self.height as f32;

                renderer.draw_line(
                    Position {
                        x: v1.x as u32,
                        y: v1.y as u32,
                    },
                    Position {
                        x: v2.x as u32,
                        y: v2.y as u32,
                    },
                    WHITE,
                );
                renderer.draw_line(
                    Position {
                        x: v2.x as u32,
                        y: v2.y as u32,
                    },
                    Position {
                        x: v3.x as u32,
                        y: v3.y as u32,
                    },
                    WHITE,
                );
                renderer.draw_line(
                    Position {
                        x: v3.x as u32,
                        y: v3.y as u32,
                    },
                    Position {
                        x: v1.x as u32,
                        y: v1.y as u32,
                    },
                    WHITE,
                );
            }
        }
    }
}

fn main() {
    let (width, height) = (256, 240);
    let mut engine = VNEngine::new_opengl(width, height, 8);
    let mut runner = Rasterizer::new(width, height);
    engine.run(&mut runner)
}